use rspc::alpha::Rspc;

const R: Rspc<()> = Rspc::new();

fn router() -> crate::Router<()> {
    R.router()
        .procedure("version", R.query(|ctx, _: ()| env!("CARGO_PKG_VERSION")))
        .compat()
}
