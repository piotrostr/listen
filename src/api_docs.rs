use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::handlers::handle_balance,
        crate::handlers::handle_token_balance,
        crate::handlers::handle_pricing,
        crate::handlers::handle_pump_buy,
        crate::handlers::handle_pump_sell,
        crate::handlers::handle_swap,
        crate::handlers::handle_get_pubkey,
        crate::handlers::handle_get_holdings
    ),
    components(schemas(
        crate::handlers::BalanceRequest,
        crate::handlers::BalanceResponse,
        crate::handlers::TokenBalanceRequest,
        crate::handlers::TokenBalanceResponse,
        crate::handlers::PriceRequest,
        crate::handlers::PriceResponse,
        crate::handlers::PumpBuyRequest,
        crate::handlers::PumpSellRequest,
        crate::handlers::SwapRequest,
        crate::handlers::HoldingsResponse,
    )),
    tags(
        (name = "balance", description = "Balance query endpoints"),
        (name = "pump-swap", description = "Pump swap endpoints"),
        (name = "swap", description = "Token swap endpoints"),
        (name = "token", description = "Token price/meta endpoints")
    )
)]
pub struct ApiDocs;
