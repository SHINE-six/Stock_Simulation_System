use redis::Commands;

const AVAILABLE_STOCKS: &[&str] = &["AAPL","GOOGL","MSFT","AMZN","NVDA","META","TSLA","CRM","ORCL","IBM","CSCO","INTC","ADBE","QCOM","AVGO","JNJ","PFE","MRK","ABT","MRNA","LLY","GILD","BMY","AMGN","UNH","CI","MDT","TMO","BIIB","REGN","JPM","BAC","WFC","C","GS","MS","AXP","BLK","SCHW","V","MA","PYPL","BRK.B","MET","PRU","PG","KO","PEP","CL","UL","PM","GM","F","NKE","MDLZ","XOM","CVX","COP","BP","TTE","MPC","SLB","HAL","BKR","PSX","BA","CAT","GE","MMM","HON","UNP","LMT","RTX","DE","NOC","NEE","DUK","D","SO","CEG","DIS","CMCSA","VZ","T","NFLX"];
const REDIS_URL: &str = "redis://localhost:6379";

fn main() {
    let client = redis::Client::open(REDIS_URL).unwrap();
    let mut conn = client.get_connection().unwrap();

    // Set the stock price for each stock to initial value of 100.0000
    for (index, stock) in AVAILABLE_STOCKS.iter().enumerate() {
        let _: () = conn.hset("stocks:prices", stock, 100.0000).unwrap();

        // Top 15 stock is in the Technology sector, the 15 healthcare, 15 finance, 10 consumer_goods, 10 energy, 10 industrial, 5 utilities, 5 communication
        let sector = match index {
            0..=14 => "Technology",
            15..=29 => "Healthcare",
            30..=44 => "Finance",
            45..=54 => "Consumer Goods",
            55..=64 => "Energy",
            65..=74 => "Industrial",
            75..=79 => "Utilities",
            80..=84 => "Communication",
            _ => "Unknown",
        };
        
        let _: () = conn.hset("stocks:sector", stock, sector).unwrap();
    }

    // Clear all hash of order_book:*
    for stock in AVAILABLE_STOCKS {
        let _: () = conn.del(format!("order_book:{}", stock)).unwrap();
    }

    println!("Stock prices initialized");
    println!("Stock sector initialized");
    println!("Order book cleared");
}
