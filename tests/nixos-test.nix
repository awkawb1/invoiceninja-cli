{ nixpkgsPath, system, invoiceninjaModule, invoiceninjaCliPkg }:

let
  pkgs = import nixpkgsPath { inherit system; };

  adminEmail = "admin@email.com";
  adminPassword = pkgs.lib.strings.trim (builtins.readFile ./invoice_ninja_test_password);
in
pkgs.testers.nixosTest {
  name = "invoiceninja-cli";

  nodes = {
    # Only one node/proxy backend: which proxy Invoice Ninja sits behind is
    # invoiceninja-nixos's own concern (its checks exercise both nginx and
    # caddy already). This test is about whether invoiceninja-cli works
    # against a real server, which doesn't depend on which proxy fronts it.
    machine = {
      imports = [ (import ./common.nix { proxyServer = "nginx"; inherit invoiceninjaModule; }) ];
      environment.systemPackages = [ invoiceninjaCliPkg ];
    };
  };

  testScript = ''
    import json

    def api_post(machine, path, payload, token=None):
        auth_header = f"-H 'X-API-TOKEN: {token}' " if token else ""
        return json.loads(machine.succeed(
            f"curl -sf -X POST http://localhost{path} "
            + auth_header
            + "-H 'Content-Type: application/json' "
            + f"-d '{json.dumps(payload)}'"
        ))

    def cli(machine, token, *args):
        quoted_args = " ".join(f"'{a}'" for a in args)
        return machine.succeed(
            f"IN_API_TOKEN={token} IN_BASE_URL=http://localhost/api/v1 "
            f"invoiceninja-cli --output json {quoted_args}"
        )

    machine.start()

    # phpfpm-invoiceninja/invoiceninja-queue-worker both Require= invoiceninja-data-setup,
    # so systemd already waits for it to finish before either starts.
    machine.wait_for_unit("mysql.service")
    machine.wait_for_unit("redis-invoiceninja.service")
    machine.wait_for_unit("phpfpm-invoiceninja.service")
    machine.wait_for_unit("invoiceninja-queue-worker.service")
    machine.wait_for_unit("nginx.service")
    machine.wait_for_open_port(80)

    with subtest("admin account can log in via the API"):
        login_response = api_post(machine, "/api/v1/login", {"email": "${adminEmail}", "password": "${adminPassword}"})
        assert "${adminEmail}" in json.dumps(login_response), f"login did not succeed: {login_response}"
        token = login_response["data"][0]["token"]["token"]

    with subtest("invoiceninja-cli can create and list a client"):
        cli(machine, token, "clients", "create", "--data", '{"name": "CLI Test Client"}')
        listing = cli(machine, token, "clients", "list")
        clients = json.loads(listing)
        assert any(c["name"] == "CLI Test Client" for c in clients), f"client not in list output: {listing}"
        client_id = next(c["id"] for c in clients if c["name"] == "CLI Test Client")

    with subtest("invoiceninja-cli update preserves fields it wasn't told about"):
        cli(machine, token, "clients", "update", client_id, "--data", '{"city": "Springfield"}')
        updated = json.loads(cli(machine, token, "clients", "get", client_id))[0]
        assert updated["city"] == "Springfield", f"city not updated: {updated}"
        assert updated["name"] == "CLI Test Client", f"update blanked a field it wasn't told about: {updated}"

    with subtest("invoiceninja-cli can create an invoice against that client and see it in a payment"):
        invoice_out = cli(
            machine, token, "invoices", "create", "--data",
            json.dumps({
                "client_id": client_id,
                "line_items": [{"quantity": 1, "cost": 100, "product_key": "test-product"}],
            }),
        )
        invoice = json.loads(invoice_out)[0]
        assert invoice["amount"] == 100, f"unexpected invoice amount: {invoice}"

        payment_out = cli(
            machine, token, "payments", "create", "--data",
            json.dumps({
                "client_id": client_id,
                "amount": invoice["amount"],
                "invoices": [{"invoice_id": invoice["id"], "amount": invoice["amount"]}],
            }),
        )
        payment = json.loads(payment_out)[0]
        assert payment["applied"] == invoice["amount"], f"payment not fully applied: {payment}"

    with subtest("invoiceninja-cli delete soft-deletes the client"):
        cli(machine, token, "clients", "delete", client_id)
        # Not `clients list`: Invoice Ninja's default /clients (no `status`
        # param) returns every record regardless of is_deleted — confirmed
        # against a real server, contrary to what CLAUDE.md's `--status`
        # doc comment implies. Check the flag directly instead of relying
        # on list-exclusion.
        deleted = json.loads(cli(machine, token, "clients", "get", client_id))[0]
        assert deleted["is_deleted"], f"delete did not set is_deleted: {deleted}"
  '';
}
