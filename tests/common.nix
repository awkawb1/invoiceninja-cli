{ proxyServer ? "nginx", invoiceninjaModule }:
{ ... }:

{
  # Unlike the invoiceninja-nixos repo this was copied from, there's no
  # sibling `nixos-module/` directory here — the module comes in through
  # this parameter instead, threaded from flake.nix as
  # `invoiceninja-nixos.nixosModules.invoiceninja` (the flake input). The
  # module's own `../package.nix` relative import inside that file still
  # resolves correctly regardless of who imports it, since it's relative to
  # the module file's own location in the Nix store, not to this directory.
  imports = [ invoiceninjaModule ];

  services.invoiceninja = {
    enable = true;
    proxy.server = proxyServer;
    adminAccount.passwordFile = ./invoice_ninja_test_password;
    secretFile = ./test.env;
    settings.APP_DEBUG = true;
  };

  networking.firewall.enable = false;
}
