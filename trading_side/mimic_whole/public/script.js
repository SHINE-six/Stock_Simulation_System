const ws = new WebSocket(`ws://${window.location.host}/ws`);

const stockTableBody = document.getElementById('stock-tbody');

// Initialize stockData from localStorage or as an empty object
let stockData = {};

// Render the table with existing data on page load
renderTable();


ws.onmessage = function(event) {
    const stock = JSON.parse(event.data);
    updateStockTable(stock);
};

function updateStockTable(stock) {
    stockData[stock.symbol] = stock.price;
    renderTable();
}

function renderTable() {
    // Clear existing rows
    stockTableBody.innerHTML = '';

    // Sorting config
    let sortColumn = "symbol";
    let direction = "asc";

    // Sort the symbols based on the selected column
    const symbols = Object.keys(stockData).sort((a, b) => {
        if (sortColumn === "symbol") {
            return direction === "asc" ? a.localeCompare(b) : b.localeCompare(a);  // sort by symbol
        } else {
            return direction === "asc" ? stockData[a] - stockData[b] : stockData[b] - stockData[a];  // sort by price
        }
    });

    for (const symbol of symbols) {
        const price = stockData[symbol];

        const row = document.createElement('tr');

        // Symbol Cell
        const symbolCell = document.createElement('td');
        symbolCell.textContent = symbol;
        row.appendChild(symbolCell);

        // Price Cell
        const priceCell = document.createElement('td');
        priceCell.textContent = price.toFixed(4);
        row.appendChild(priceCell);

        stockTableBody.appendChild(row);
    }
}

const selectSymbol = document.getElementById('symbol-option');
const refreshSymbolOption = document.getElementById('refresh-symbol-option');

const buy_sell_action = document.getElementById('buy-sell-select');

const quantity = document.getElementById('quantity-input');

const price = document.getElementById('price-input');

const placeOrderButton = document.getElementById('place-order-button');

// detect if selectSymbol is opened
refreshSymbolOption.addEventListener('click', function() {
        // clear all options
        selectSymbol.innerHTML = '';
        addSymbolOption();
        console.log('clicked');
    });
    
addSymbolOption();

function addSymbolOption() {
    // add a dropdown of all symbols
    const select = document.createElement('select');
    select.name = 'symbol';
    select.id = 'symbol';
    select.required = true;
    for (const symbol of Object.keys(stockData)) {
        const option = document.createElement('option');
        option.value = symbol;
        option.textContent = symbol;
        select.appendChild(option);
    }

    // // dummy data
    // const option1 = document.createElement('option');
    // option1.value = 'AAPL';
    // option1.textContent = 'AAPL';
    // select.appendChild(option1);

    // const option2 = document.createElement('option');
    // option2.value = 'AMZN';
    // option2.textContent = 'AMZN';
    // select.appendChild(option2);


    selectSymbol.appendChild(select);
}



placeOrderButton.addEventListener('click', function() {
    const symbol = document.getElementById('symbol').value;
    const action = buy_sell_action.value;
    const qty = quantity.value;
    const p = price.value;

    // Make sure all fields are filled
    if (!symbol || !action || !qty || !p) {
        alert('Please fill in all fields');
        return;
        // Make sure the qty is positive integer, and price is positive float
    } else if (!Number.isInteger(parseFloat(qty)) || qty <= 0) {
        alert('Please enter a positive integer for quantity');
        return;
    }
    else if (parseFloat(p) <= 0) {
        alert('Please enter a positive float for price');
        return;
    }

    console.log(symbol, action, qty, p);
    
    // send order to server at /order
    fetch('/order', {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
        },
        body: JSON.stringify({
            stock_symbol: symbol,
            order_type: action,
            quantity: qty,
            price: p
        }),
    })
    .then(response => response.json())
    .then(data => {
        console.log('Success:', data);
        alert('Order placed successfully');
    })
});