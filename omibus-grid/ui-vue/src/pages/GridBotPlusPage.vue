<template>
  <div class="gbp-root">

    <!-- Messages -->
    <div id="gbpMessages" class="messages-container"></div>

    <!-- Header -->
    <div class="page-header">
      <div class="page-title">🤖 GridBot+ Full Complete Form</div>
      <small style="color:rgba(255,255,255,0.6)">with DemoGridBot &amp; QuickActionsPanel</small>
    </div>

    <!-- Form Card -->
    <div class="form-card">
      <div class="form-title">⚙️ Create New Grid Bot</div>

      <div class="form-row">
        <div class="form-group">
          <label>Bot Name</label>
          <input type="text" id="gbp_botName" placeholder="Bot name" />
        </div>
        <div class="form-group">
          <label>Symbol</label>
          <select id="gbp_symbol">
            <option>BTC/USDC</option>
            <option>ETH/USDC</option>
            <option>LCX/USDC</option>
          </select>
        </div>
        <div class="form-group">
          <label>Exchange</label>
          <select id="gbp_exchange">
            <option>coinbaseadvanced</option>
            <option>kraken</option>
            <option>lcx</option>
          </select>
        </div>
      </div>

      <!-- Price Grid with Buttons -->
      <div class="form-row">
        <div class="form-group">
          <label>Lower Price</label>
          <input type="number" id="gbp_lowerPrice" placeholder="60000" step="0.01" />
          <div class="price-buttons">
            <button class="price-btn" @click="gbp_adjustLowerPrice(-0.9)">-90%</button>
            <button class="price-btn" @click="gbp_adjustLowerPrice(-0.7)">-70%</button>
            <button class="price-btn" @click="gbp_adjustLowerPrice(-0.5)">-50%</button>
            <button class="price-btn" @click="gbp_adjustLowerPrice(-0.3)">-30%</button>
            <button class="price-btn" @click="gbp_adjustLowerPrice(-0.2)">-20%</button>
            <button class="price-btn" @click="gbp_adjustLowerPrice(-0.1)">-10%</button>
            <button class="price-btn" @click="gbp_adjustLowerPrice(-0.05)">-5%</button>
            <button class="price-btn" @click="gbp_adjustLowerPrice(-0.01)">-1%</button>
          </div>
        </div>
        <div class="form-group">
          <label>Upper Price</label>
          <input type="number" id="gbp_upperPrice" placeholder="80000" step="0.01" />
          <div class="price-buttons">
            <button class="price-btn" @click="gbp_adjustUpperPrice(0.01)">+1%</button>
            <button class="price-btn" @click="gbp_adjustUpperPrice(0.05)">+5%</button>
            <button class="price-btn" @click="gbp_adjustUpperPrice(0.1)">+10%</button>
            <button class="price-btn" @click="gbp_adjustUpperPrice(0.2)">+20%</button>
            <button class="price-btn" @click="gbp_adjustUpperPrice(0.3)">+30%</button>
            <button class="price-btn" @click="gbp_adjustUpperPrice(0.5)">+50%</button>
            <button class="price-btn" @click="gbp_adjustUpperPrice(0.7)">+70%</button>
            <button class="price-btn" @click="gbp_adjustUpperPrice(0.9)">+90%</button>
          </div>
        </div>
        <div class="form-group">
          <label>Grid Count</label>
          <input type="number" id="gbp_grids" value="20" min="1" max="200" />
        </div>
        <div class="form-group">
          <label>Amount</label>
          <input type="number" id="gbp_amount" placeholder="1000" step="0.01" />
        </div>
      </div>

      <!-- Live Prices -->
      <div class="header-prices">
        <div class="price-badge bid">
          <div>BID</div>
          <div id="gbp_bestBidDisplay">-</div>
        </div>
        <div class="price-badge ask">
          <div>ASK</div>
          <div id="gbp_bestAskDisplay">-</div>
        </div>
      </div>

      <!-- Config -->
      <div class="form-row">
        <div class="form-group">
          <label>Amount Type</label>
          <select id="gbp_amountType">
            <option value="quantityPerGrid">Quantity Per Grid</option>
            <option value="totalAmount">Total Amount</option>
            <option value="incrementalPercent">Incremental %</option>
          </select>
        </div>
        <div class="form-group">
          <label>Orders Side</label>
          <select id="gbp_ordersSide">
            <option value="buyOrSell">Buy &amp; Sell</option>
            <option value="buyOnly">Buy Only</option>
            <option value="sellOnly">Sell Only</option>
          </select>
        </div>
      </div>

      <!-- BUY/SELL Table -->
      <div style="overflow-x:auto">
        <table class="buy-sell-table">
          <tbody>
            <tr>
              <td class="buy-cell header-row">📈 BUY</td>
              <td class="sell-cell header-row">📉 SELL</td>
            </tr>
            <tr>
              <td class="buy-cell">INC %</td>
              <td class="sell-cell">INC %</td>
            </tr>
            <tr>
              <td class="buy-cell"><input type="number" id="gbp_incBuy" value="1" min="0" step="0.1" /></td>
              <td class="sell-cell"><input type="number" id="gbp_incSell" value="1" min="0" step="0.1" /></td>
            </tr>
            <tr>
              <td class="buy-cell">DEV PRICE %</td>
              <td class="sell-cell">DEV PRICE %</td>
            </tr>
            <tr>
              <td class="buy-cell"><input type="number" id="gbp_devPriceBuy" value="1" min="0" step="0.1" /></td>
              <td class="sell-cell"><input type="number" id="gbp_devPriceSell" value="1" min="0" step="0.1" /></td>
            </tr>
            <tr>
              <td class="buy-cell">DEV AMOUNT %</td>
              <td class="sell-cell">DEV AMOUNT %</td>
            </tr>
            <tr>
              <td class="buy-cell"><input type="number" id="gbp_devAmtBuy" value="0.9" min="0" step="0.1" /></td>
              <td class="sell-cell"><input type="number" id="gbp_devAmtSell" value="0.9" min="0" step="0.1" /></td>
            </tr>
          </tbody>
        </table>
      </div>

      <!-- Buttons -->
      <div class="form-actions">
        <button class="btn btn-primary" @click="gbp_createBot()">⚙️ CREATE BOT</button>
        <button class="btn btn-demo"    @click="gbp_toggleDemoGrid()">🎯 VIEW DEMO GRID</button>
        <button class="btn btn-lib"     @click="gbp_toggleLibraryDocs()">📚 LIBRARY &amp; ENGINE</button>
        <button class="btn btn-reset"   @click="gbp_resetForm()">🔄 RESET</button>
      </div>
    </div>

    <!-- Active Bots Section -->
    <div class="bots-section">
      <h3 style="color:#10eb04;margin-bottom:16px">📊 Active Bots</h3>
      <div class="bots-grid" id="gbp_botsList">
        <div style="text-align:center;padding:40px;color:rgba(255,255,255,0.5)">
          🤖 No active bots yet
        </div>
      </div>
    </div>

    <!-- Quick Actions Panel (Draggable) -->
    <div class="qa-panel" id="gbp_qaPanel">
      <div class="qa-header" @mousedown="gbp_dragQAPanel($event)">
        <span>💰 Quick Actions</span>
        <button class="x-btn" @click="gbp_toggleQAPanel()">✕</button>
      </div>
      <div class="qa-buttons">
        <button class="qa-btn" @click="gbp_quickAdjustLower(-0.2)">Lower -20%</button>
        <button class="qa-btn" @click="gbp_quickAdjustLower(-0.1)">Lower -10%</button>
        <button class="qa-btn" @click="gbp_quickAdjustUpper(0.1)">Upper +10%</button>
        <button class="qa-btn" @click="gbp_quickAdjustUpper(0.2)">Upper +20%</button>
        <button class="qa-btn" @click="gbp_quickAddGrids(5)">Grids +5</button>
        <button class="qa-btn" @click="gbp_quickSubGrids(5)">Grids -5</button>
        <button class="qa-btn" @click="gbp_quickAddAmount(100)">Amt +100</button>
        <button class="qa-btn" @click="gbp_quickSubAmount(100)">Amt -100</button>
      </div>
    </div>

    <!-- Demo Grid Modal (Draggable) -->
    <div class="demo-modal hidden" id="gbp_demoModal">
      <div class="demo-header" @mousedown="gbp_dragDemoModal($event)">
        <span>🎯 Demo Grid Preview</span>
        <button class="x-btn" @click="gbp_toggleDemoGrid()">✕</button>
      </div>
      <div class="demo-stats">
        <div class="stat-box"><div class="stat-label">Total Orders</div><div class="stat-value" id="gbp_demoTotalOrders">0</div></div>
        <div class="stat-box"><div class="stat-label">Buy Orders</div><div class="stat-value" style="color:#10eb04" id="gbp_demoBuyOrders">0</div></div>
        <div class="stat-box"><div class="stat-label">Sell Orders</div><div class="stat-value" style="color:#eb0404" id="gbp_demoSellOrders">0</div></div>
        <div class="stat-box"><div class="stat-label">Avg Price</div><div class="stat-value" id="gbp_demoAvgPrice">-</div></div>
        <div class="stat-box"><div class="stat-label">Total Value</div><div class="stat-value" id="gbp_demoTotalValue">-</div></div>
        <div class="stat-box"><div class="stat-label">Invest</div><div class="stat-value" id="gbp_demoInvest">-</div></div>
      </div>
      <div style="font-size:10px;color:#3b82f6;font-weight:700;margin-bottom:8px;text-transform:uppercase">Grid Orders</div>
      <div class="demo-orders" id="gbp_demoOrdersList">
        <div style="text-align:center;padding:20px;color:rgba(255,255,255,0.5)">Fill form and click "VIEW DEMO GRID" to see orders</div>
      </div>
    </div>

    <!-- GridBot Library & Engine Documentation Modal -->
    <div class="library-modal hidden" id="gbp_libraryModal">
      <div class="library-header" @mousedown="gbp_dragLibraryModal($event)">
        <span>📚 GridBot Library &amp; Engine Reference</span>
        <button class="x-btn" @click="gbp_toggleLibraryDocs()">✕</button>
      </div>
      <div class="library-content">
        <div class="doc-section">
          <h3>🎯 Core Purpose</h3>
          <p>GridBotLib implements the complete grid trading algorithm. GridBotEngine runs it on a 1-second interval to monitor filled orders and create inverse orders automatically.</p>
        </div>

        <div class="doc-section">
          <h3>💻 Language Implementation Tabs</h3>
          <div style="margin-top:12px">
            <div style="display:flex;gap:8px;margin-bottom:16px;border-bottom:1px solid rgba(100,200,255,0.2);padding-bottom:12px">
              <button @click="gbp_switchLangTab('js', $event)"  class="lang-tab active-tab">📘 JavaScript</button>
              <button @click="gbp_switchLangTab('cpp', $event)" class="lang-tab">⚙️ C++</button>
              <button @click="gbp_switchLangTab('zig', $event)" class="lang-tab">⚡ Zig</button>
            </div>

            <div id="gbp_lang-js" class="lang-content">
              <pre>// JavaScript: botLibreri.js
import { create, all } from 'mathjs';

export class GridBotLibreri {
  constructor(exchangeAPI, database) {
    this.exchangeAPI = exchangeAPI;
    this.database = database;
  }

  async createBot(params) {
    const { userID, exchange, symbol, lowerPrice, upperPrice, gridCount } = params;

    // Calculate grid prices
    const gridWidth = (upperPrice - lowerPrice) / gridCount;
    let prices = [];
    for (let i = 0; i &lt; gridCount; i++) {
      prices.push(lowerPrice + (i + 1) * gridWidth);
    }

    // Place orders
    const buyPrices = prices.filter(p => p &lt; marketPrice);
    const sellPrices = prices.filter(p => p > marketPrice);

    let allOrders = [];
    allOrders.push(...await this.placeBuyOrders(...));
    allOrders.push(...await this.placeSellOrders(...));

    return { _id: botId, activeOrders: allOrders, status: 'running' };
  }

  async checkAndPlaceOrder(dbOrder, exchangeOrder, bot) {
    if (exchangeOrder.status === 'closed') {
      const invPrice = this._calcInversePrice(exchangeOrder.price);
      const invAmount = this._calcInverseAmount(exchangeOrder.amount);
      return await this.exchangeAPI.createOrder(...);
    }
  }
}</pre>
            </div>

            <div id="gbp_lang-cpp" class="lang-content" style="display:none">
              <pre>// C++: GridBotLibreri.hpp
#include &lt;vector&gt;
#include &lt;boost/multiprecision/cpp_dec_float.hpp&gt;

using decimal = boost::multiprecision::cpp_dec_float_50;

class GridBotLibreri {
private:
  ExchangeAPI* exchangeAPI;
  Database* database;

public:
  GridBotLibreri(ExchangeAPI* api, Database* db)
    : exchangeAPI(api), database(db) {}

  struct Bot {
    std::string id;
    std::vector&lt;Order&gt; activeOrders;
    std::string status;
  };

  Bot createBot(const BotParams&amp; params) {
    decimal gridWidth = (params.upperPrice - params.lowerPrice)
                      / params.gridCount;

    std::vector&lt;decimal&gt; prices;
    for (int i = 0; i &lt; params.gridCount; ++i) {
      prices.push_back(params.lowerPrice + (i + 1) * gridWidth);
    }

    std::vector&lt;Order&gt; allOrders;
    for (auto price : prices) {
      if (price &lt; marketPrice) {
        allOrders.push_back(placeBuyOrder(price, params));
      } else {
        allOrders.push_back(placeSellOrder(price, params));
      }
    }

    Bot bot;
    bot.activeOrders = allOrders;
    bot.status = "running";
    return bot;
  }

  bool checkAndPlaceOrder(const Order&amp; dbOrder,
                         const Order&amp; exOrder,
                         const Bot&amp; bot) {
    if (exOrder.status == "closed") {
      decimal invPrice = calcInversePrice(exOrder.price);
      decimal invAmount = calcInverseAmount(exOrder.amount);
      return exchangeAPI-&gt;createOrder(invPrice, invAmount);
    }
    return false;
  }
};</pre>
            </div>

            <div id="gbp_lang-zig" class="lang-content" style="display:none">
              <pre>// Zig: gridbot_libreri.zig
const std = @import("std");
const Decimal = @import("decimal.zig").Decimal;

pub const GridBotLibreri = struct {
    exchangeAPI: *ExchangeAPI,
    database: *Database,
    allocator: std.mem.Allocator,

    pub fn init(api: *ExchangeAPI, db: *Database,
                allocator: std.mem.Allocator) GridBotLibreri {
        return GridBotLibreri{
            .exchangeAPI = api,
            .database = db,
            .allocator = allocator,
        };
    }

    pub fn createBot(self: *GridBotLibreri,
                     params: BotParams) !Bot {
        var arena = std.heap.ArenaAllocator.init(self.allocator);
        defer arena.deinit();

        const gridWidth = (params.upperPrice - params.lowerPrice)
                        / @intToFloat(f64, params.gridCount);

        var prices = std.ArrayList(Decimal).init(arena.allocator());
        for (0..params.gridCount) |i| {
            const price = params.lowerPrice
                        + (@intToFloat(f64, i + 1) * gridWidth);
            try prices.append(Decimal.from(price));
        }

        var allOrders = std.ArrayList(Order).init(arena.allocator());

        for (prices.items) |price| {
            if (price &lt; marketPrice) {
                try allOrders.append(
                    try self.placeBuyOrder(price, params)
                );
            } else {
                try allOrders.append(
                    try self.placeSellOrder(price, params)
                );
            }
        }

        return Bot{
            .id = try generateId(arena.allocator()),
            .activeOrders = allOrders.items,
            .status = "running",
        };
    }

    pub fn checkAndPlaceOrder(self: *GridBotLibreri,
                              dbOrder: Order,
                              exOrder: Order,
                              bot: *Bot) !bool {
        if (std.mem.eql(u8, exOrder.status, "closed")) {
            const invPrice = self.calcInversePrice(exOrder.price);
            const invAmount = self.calcInverseAmount(exOrder.amount);
            return try self.exchangeAPI.createOrder(invPrice, invAmount);
        }
        return false;
    }
};</pre>
            </div>
          </div>
        </div>

        <div class="doc-section">
          <h3>⚙️ Key Functions</h3>

          <div class="function-doc">
            <h4>createBot(data)</h4>
            <p><strong>Purpose:</strong> Initialize bot, calculate grid prices, place initial orders</p>
            <p><strong>Inputs:</strong> userID, exchange, symbol, lowerPrice, upperPrice, nrOfGrids, amountType, amount, ordersSide</p>
            <p><strong>Process:</strong></p>
            <ul>
              <li>Fetch balance snapshot (base &amp; quote tokens)</li>
              <li>Calculate gridWidth = (upperPrice - lowerPrice) / nrOfGrids</li>
              <li>Generate buy prices: [lowerPrice, lowerPrice+gridWidth, ...]</li>
              <li>Generate sell prices: [upperPrice, upperPrice-gridWidth, ...]</li>
              <li>Split orders: if buyOrSell → 50% buy, 50% sell</li>
              <li>Call placeBuyOrders() and placeSellOrders()</li>
              <li>Store bot in DB with activeOrders array</li>
            </ul>
            <p><strong>Returns:</strong> Bot object with activeOrders array (exchange IDs)</p>
          </div>

          <div class="function-doc">
            <h4>placeBuyOrders(userID, exchange, symbol, buyPrices, amountType, amount, nrOfGrids, incrementalPercent, apiKeyName)</h4>
            <p><strong>Purpose:</strong> Create all buy limit orders</p>
            <p><strong>Logic:</strong></p>
            <ul>
              <li>Loop through buyPrices array</li>
              <li>For each price, calculate quantityPerGrid using getQuantityPerGrid()</li>
              <li>Place limit order: amount=quantityPerGrid, price=buyPrice</li>
              <li>Collect exchange response IDs</li>
            </ul>
            <p><strong>Returns:</strong> Array of order objects with {{'{'}}id, price, amount, status: 'open'{{'}'}}</p>
          </div>

          <div class="function-doc">
            <h4>getQuantityPerGrid(price, amountType, amount, nrOfGrids, incrementalPercent, index)</h4>
            <p><strong>Purpose:</strong> Calculate order amount based on 3 modes</p>
            <p><strong>Modes:</strong></p>
            <ul>
              <li><strong>quantityPerGrid:</strong> quantity = amount / price</li>
              <li><strong>totalAmount:</strong> quantity = (amount / nrOfGrids) / price</li>
              <li><strong>incrementalPercent:</strong> quantity = (amount + ((amount/100) * (incrementalPercent * index))) / price</li>
            </ul>
            <p><strong>Returns:</strong> BigNumber quantity (use mathjs for precision)</p>
          </div>

          <div class="function-doc">
            <h4>checkAndPlaceOrder(dbOrder, order, bot, gridOrdersIndex)</h4>
            <p><strong>Purpose:</strong> Detect filled order and create inverse order</p>
            <p><strong>Logic:</strong></p>
            <ul>
              <li>Match order IDs: dbOrder.id === order.id</li>
              <li>Check if order.status === 'closed' (FILLED)</li>
              <li>Calculate inverse order amount with deviation: amount * devAmount%</li>
              <li>Apply price adjustment: inverseSide uses deviation price%</li>
              <li>Count buy/sell fills to balance grid</li>
              <li>Create limit order for inverse side</li>
              <li>Update bot.activeOrders in DB</li>
            </ul>
            <p><strong>Example:</strong> BUY filled → place SELL above fill price with deviation</p>
          </div>

          <div class="function-doc">
            <h4>calculateBalanceInOrders(orders)</h4>
            <p><strong>Purpose:</strong> Sum total balance locked in active orders</p>
            <p><strong>Logic:</strong></p>
            <ul>
              <li>For buy orders: sum (amount * price)</li>
              <li>For sell orders: sum (amount)</li>
              <li>Return {{'{'}} buyTotal, sellTotal, grandTotal {{'}'}}</li>
            </ul>
            <p><strong>Returns:</strong> Balance breakdown in quote token</p>
          </div>
        </div>

        <div class="doc-section">
          <h3>🔄 Deviation Adjustments</h3>
          <p>When creating inverse orders:</p>
          <ul>
            <li><strong>devPriceBuy/Sell:</strong> Offset price by ±X% from market</li>
            <li><strong>devAmountBuy/Sell:</strong> Scale order amount by ±X%</li>
            <li><strong>incBuy/incSell:</strong> Incremental % increase per grid level</li>
          </ul>
          <p><strong>Example:</strong> Fill buy order, incSell=1.1 → next sell order uses 110% of base amount</p>
        </div>

        <div class="doc-section">
          <h3>⏱️ GridBotEngine (1-second Scheduler)</h3>
          <p><strong>Runs every 1 second:</strong></p>
          <ol>
            <li>Query DB: fetch all bots with status='running'</li>
            <li>For each bot:</li>
            <li>&nbsp;&nbsp;a) Call ccxt.fetchClosedOrders() → get filled orders since last check</li>
            <li>&nbsp;&nbsp;b) For each filled order → call checkAndPlaceOrder()</li>
            <li>&nbsp;&nbsp;c) Update bot.activeOrders array in DB</li>
            <li>&nbsp;&nbsp;d) Update bot.filledOrders counter</li>
            <li>Sleep 1000ms, repeat</li>
          </ol>
          <p><strong>Key:</strong> Maintains perpetual grid: fill = inverse order automatically</p>
        </div>

        <div class="doc-section">
          <h3>💾 Database Schema Fields</h3>
          <p><strong>Bot Document:</strong></p>
          <ul>
            <li>userID, exchange, symbol</li>
            <li>lowerPrice, upperPrice, nrOfGrids</li>
            <li>amountType, amount, ordersSide</li>
            <li>incBuy, incSell (incremental %)</li>
            <li>devPriceBuy, devPriceSell, devAmountBuy, devAmountSell</li>
            <li>activeOrders: [{{'{'}}id, price, amount, side, status{{'}'}}]</li>
            <li>filledOrders: count</li>
            <li>status: 'running' | 'paused' | 'stopped'</li>
            <li>balanceSnapshot: {{'{'}}baseFree, baseTotal, quoteFree, quoteTotal{{'}'}}</li>
          </ul>
        </div>

        <div class="doc-section">
          <h3>📋 Data Flow Diagram</h3>
          <pre>User fills form (prices, grids, amounts)
    ↓
createBot(data) called
    ↓
Calculate grid prices array
    ↓
placeBuyOrders() + placeSellOrders()
    ↓
Orders sent to exchange (limit orders)
    ↓
GridBotEngine (1s loop)
    ├→ Fetch closed orders from exchange
    ├→ For each filled: checkAndPlaceOrder()
    ├→ Create inverse order (opposite side)
    └→ Update DB activeOrders array
    ↓
Cycle repeats (perpetual grid)</pre>
        </div>

        <div class="doc-section">
          <h3>📚 SECTION 1: Math Functions (Language-Agnostic)</h3>

          <div class="function-doc">
            <h4>calculateGridPrices(lowerPrice, upperPrice, gridCount)</h4>
            <pre>// Generate all grid price levels
const prices = calculateGridPrices(60000, 80000, 20);
// Returns: [61000, 62000, ..., 80000]
// gridWidth = (upperPrice - lowerPrice) / gridCount
// prices = [lower + width, lower + 2*width, ..., upper]</pre>
          </div>

          <div class="function-doc">
            <h4>splitPricesBySide(allPrices, marketPrice)</h4>
            <pre>// Split prices into buy/sell by market position
const {buyPrices, sellPrices} = splitPricesBySide(
    [61000, 62000, ..., 80000], 70000
);
// buyPrices:  [61000..69000]  (below market)
// sellPrices: [71000..80000]  (above market)</pre>
          </div>

          <div class="function-doc">
            <h4>getQuantityPerGrid(price, amountType, baseAmount, gridCount, incPercent, index)</h4>
            <pre>// Three modes:
// 1. 'quantityPerGrid':    qty = baseAmount / price
// 2. 'totalAmount':        qty = (baseAmount / gridCount) / price
// 3. 'incrementalPercent': qty = (base + (base/100)*percent*index) / price</pre>
          </div>

          <div class="function-doc">
            <h4>calculateInversePrice(filledPrice, filledSide, devPercent)</h4>
            <pre>// If filled SELL: price - ((price/100) * dev)
// If filled BUY:  price + ((price/100) * dev)</pre>
          </div>

          <div class="function-doc">
            <h4>calculateInverseAmount(filledAmount, filledSide, devPercent)</h4>
            <pre>// Returns: amount + ((amount/100) * devPercent)
// 90% = 0.9x, 100% = 1.0x, 110% = 1.1x</pre>
          </div>

          <div class="function-doc">
            <h4>calculateBalanceInOrders(activeOrders)</h4>
            <pre>// BUY locks:  price * amount  (quote token)
// SELL locks: amount          (base token)
// Returns: {baseInOrders, quoteInOrders}</pre>
          </div>
        </div>

        <div class="doc-section">
          <h3>📖 botEngine.js — The Perpetual Grid Scheduler</h3>

          <div class="function-doc">
            <h4>Pseudo-Code: Main Loop</h4>
            <pre>const libreri = new GridBotLibreri(exchangeAPI, database);

setInterval(async () => {
    const runningBots = await database.getRunningBots();

    for (const bot of runningBots) {
        const exchangeOrders = await exchangeAPI.fetchClosedOrders(
            bot.userID, bot.exchange, bot.symbol
        );

        for (const dbOrder of bot.activeOrders) {
            for (const exOrder of exchangeOrders) {
                if (dbOrder.id === exOrder.id &amp;&amp; exOrder.status === 'closed') {
                    await libreri.checkAndPlaceOrder(dbOrder, exOrder, bot);
                }
            }
        }

        await database.saveBotDoc(bot);
    }
}, 1000); // 1 second interval</pre>
          </div>
        </div>

        <div class="doc-section">
          <h3>🔧 Porting to Other Languages</h3>
          <p><strong>Critical Precision:</strong> Use BigNumber/Decimal (20 decimals) for ALL price/amount math</p>
          <p><strong>Core Algorithm:</strong></p>
          <ol>
            <li>Calculate grid: gridWidth = (upper - lower) / count</li>
            <li>Generate prices: loop from lower to upper, step by gridWidth</li>
            <li>For each price, call getQuantityPerGrid() with index</li>
            <li>Place orders (buy side, then sell side)</li>
            <li>1s loop: fetch closed orders, match by ID, create inverse if filled</li>
          </ol>
        </div>
      </div>
    </div>

  </div>
</template>

<script setup lang="ts">
import { onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useMarket } from '../composables/useMarket'

let gbp_activeBots: any[] = []
let gbp_bestBid: number | null = null
let gbp_bestAsk: number | null = null
let gbp_qaX = 0, gbp_qaY = 0, gbp_demoX = 0, gbp_demoY = 0, gbp_libX = 0, gbp_libY = 0
let gbp_priceTimer: ReturnType<typeof setInterval>

function gbp_$(id: string) { return document.getElementById(id) as HTMLInputElement | null }
function gbp_val(id: string) { return parseFloat((gbp_$(id) as HTMLInputElement)?.value || '0') || 0 }

let gbp_exchangeReady = false
async function gbp_updateLivePrices() {
  const { exchange: mktEx, symbol: mktSym } = useMarket()
  const ex  = mktEx.value  || 'kraken'
  const sym = mktSym.value || 'XBT/USD'
  if (!gbp_exchangeReady) {
    try { await invoke('exchange_register_public', { exchange: ex }); gbp_exchangeReady = true } catch {}
  }
  try {
    const t = await invoke('exchange_ticker', { exchange: ex, symbol: sym }) as any
    if (t?.bid) gbp_bestBid = t.bid
    if (t?.ask) gbp_bestAsk = t.ask
  } catch {
    // fallback: try orderbook
    try {
      const ob = await invoke('exchange_orderbook', { exchange: ex, symbol: sym, depth: 1 }) as any
      if (ob?.bids?.[0]?.[0]) gbp_bestBid = ob.bids[0][0]
      if (ob?.asks?.[0]?.[0]) gbp_bestAsk = ob.asks[0][0]
    } catch {}
  }
  const b = gbp_$('gbp_bestBidDisplay'); if (b) b.textContent = gbp_bestBid ? gbp_bestBid.toFixed(5) : '--'
  const a = gbp_$('gbp_bestAskDisplay'); if (a) a.textContent = gbp_bestAsk ? gbp_bestAsk.toFixed(5) : '--'
}

function gbp_adjustLowerPrice(pct: number) {
  if (!gbp_bestBid) return gbp_showMessage('⚠️ BID price not available', 'error')
  const p = gbp_bestBid * (1 + pct)
  const el = gbp_$('gbp_lowerPrice'); if (el) el.value = p.toFixed(2)
  gbp_showMessage(`📉 Lower: ${p.toFixed(2)}`, 'info')
  gbp_updateDemoGrid()
}

function gbp_adjustUpperPrice(pct: number) {
  if (!gbp_bestAsk) return gbp_showMessage('⚠️ ASK price not available', 'error')
  const p = gbp_bestAsk * (1 + pct)
  const el = gbp_$('gbp_upperPrice'); if (el) el.value = p.toFixed(2)
  gbp_showMessage(`📈 Upper: ${p.toFixed(2)}`, 'info')
  gbp_updateDemoGrid()
}

function gbp_quickAdjustLower(pct: number) { gbp_adjustLowerPrice(pct) }
function gbp_quickAdjustUpper(pct: number) { gbp_adjustUpperPrice(pct) }

function gbp_quickAddGrids(val: number) {
  const el = gbp_$('gbp_grids'); if (el) el.value = String(parseInt(el.value) + val)
  gbp_updateDemoGrid()
}
function gbp_quickSubGrids(val: number) {
  const el = gbp_$('gbp_grids'); if (el) el.value = String(Math.max(1, parseInt(el.value) - val))
  gbp_updateDemoGrid()
}
function gbp_quickAddAmount(val: number) {
  const el = gbp_$('gbp_amount'); if (el) el.value = String((parseFloat(el.value) || 0) + val)
  gbp_updateDemoGrid()
}
function gbp_quickSubAmount(val: number) {
  const el = gbp_$('gbp_amount'); if (el) el.value = String(Math.max(0, (parseFloat(el.value) || 0) - val))
  gbp_updateDemoGrid()
}

function gbp_updateDemoGrid() {
  const lower = gbp_val('gbp_lowerPrice')
  const upper = gbp_val('gbp_upperPrice')
  const grids = gbp_val('gbp_grids') || 0
  const amount = gbp_val('gbp_amount')
  const amountType = (gbp_$('gbp_amountType') as HTMLSelectElement)?.value || 'quantityPerGrid'

  const list = gbp_$('gbp_demoOrdersList')
  if (!lower || !upper || lower >= upper || !grids) {
    if (list) list.innerHTML = '<div style="text-align:center;padding:20px;color:rgba(255,255,255,0.5)">Fill all fields</div>'
    return
  }

  const current = gbp_bestBid && gbp_bestAsk ? (gbp_bestBid + gbp_bestAsk) / 2 : (lower + upper) / 2
  const priceStep = (upper - lower) / grids
  let orders: any[] = []
  let totalValue = 0

  for (let i = 0; i < grids; i++) {
    const price = lower + i * priceStep
    const qty = amountType === 'totalAmount' ? (amount / grids) / price : amount / price
    const side = price < current ? 'BUY' : 'SELL'
    const value = qty * price
    totalValue += value
    orders.push({ price: price.toFixed(2), qty: qty.toFixed(4), value: value.toFixed(2), side })
  }

  const buyOrders = orders.filter(o => o.side === 'BUY')
  const sellOrders = orders.filter(o => o.side === 'SELL')
  const avgPrice = orders.reduce((s, o) => s + parseFloat(o.price), 0) / orders.length

  const set = (id: string, v: string) => { const el = gbp_$(id); if (el) el.textContent = v }
  set('gbp_demoTotalOrders', String(orders.length))
  set('gbp_demoBuyOrders', String(buyOrders.length))
  set('gbp_demoSellOrders', String(sellOrders.length))
  set('gbp_demoAvgPrice', avgPrice.toFixed(2))
  set('gbp_demoTotalValue', totalValue.toFixed(2))
  set('gbp_demoInvest', (amount * grids).toFixed(2))

  if (list) list.innerHTML = orders.map((o, i) => `
    <div class="demo-order-row ${o.side.toLowerCase()}">
      <div>#${i}</div><div>${o.side}</div><div>${o.price}</div><div>${o.qty}</div><div>$${o.value}</div>
    </div>`).join('')
}

function gbp_toggleDemoGrid() {
  const modal = gbp_$('gbp_demoModal')
  if (!modal) return
  modal.classList.toggle('hidden')
  if (!modal.classList.contains('hidden')) gbp_updateDemoGrid()
}

function gbp_toggleQAPanel() {
  gbp_$('gbp_qaPanel')?.classList.toggle('hidden')
}

function gbp_toggleLibraryDocs() {
  gbp_$('gbp_libraryModal')?.classList.toggle('hidden')
}

function gbp_dragQAPanel(e: MouseEvent) {
  const startX = e.clientX - gbp_qaX, startY = e.clientY - gbp_qaY
  const move = (e: MouseEvent) => {
    gbp_qaX = e.clientX - startX; gbp_qaY = e.clientY - startY
    const el = gbp_$('gbp_qaPanel'); if (el) { el.style.left = gbp_qaX + 'px'; el.style.top = gbp_qaY + 'px' }
  }
  const up = () => { document.removeEventListener('mousemove', move); document.removeEventListener('mouseup', up) }
  document.addEventListener('mousemove', move); document.addEventListener('mouseup', up)
}

function gbp_dragDemoModal(e: MouseEvent) {
  const startX = e.clientX - gbp_demoX, startY = e.clientY - gbp_demoY
  const move = (e: MouseEvent) => {
    gbp_demoX = e.clientX - startX; gbp_demoY = e.clientY - startY
    const el = gbp_$('gbp_demoModal')
    if (el) { el.style.left = gbp_demoX + 'px'; el.style.top = gbp_demoY + 'px'; el.style.transform = 'none' }
  }
  const up = () => { document.removeEventListener('mousemove', move); document.removeEventListener('mouseup', up) }
  document.addEventListener('mousemove', move); document.addEventListener('mouseup', up)
}

function gbp_dragLibraryModal(e: MouseEvent) {
  const startX = e.clientX - gbp_libX, startY = e.clientY - gbp_libY
  const move = (e: MouseEvent) => {
    gbp_libX = e.clientX - startX; gbp_libY = e.clientY - startY
    const el = gbp_$('gbp_libraryModal')
    if (el) { el.style.left = gbp_libX + 'px'; el.style.top = gbp_libY + 'px'; el.style.transform = 'none' }
  }
  const up = () => { document.removeEventListener('mousemove', move); document.removeEventListener('mouseup', up) }
  document.addEventListener('mousemove', move); document.addEventListener('mouseup', up)
}

function gbp_createBot() {
  const nameEl = gbp_$('gbp_botName') as HTMLInputElement
  const botData = {
    id: 'bot_' + Date.now(),
    name: nameEl?.value || `Grid_${Date.now().toString().slice(-5)}`,
    symbol: (gbp_$('gbp_symbol') as HTMLSelectElement)?.value,
    exchange: (gbp_$('gbp_exchange') as HTMLSelectElement)?.value,
    lowerPrice: gbp_val('gbp_lowerPrice'),
    upperPrice: gbp_val('gbp_upperPrice'),
    grids: gbp_val('gbp_grids'),
    amount: gbp_val('gbp_amount'),
    amountType: (gbp_$('gbp_amountType') as HTMLSelectElement)?.value,
    incBuy: gbp_val('gbp_incBuy'),
    incSell: gbp_val('gbp_incSell'),
    devPriceBuy: gbp_val('gbp_devPriceBuy'),
    devPriceSell: gbp_val('gbp_devPriceSell'),
    devAmtBuy: gbp_val('gbp_devAmtBuy'),
    devAmtSell: gbp_val('gbp_devAmtSell'),
    status: 'running'
  }

  if (!botData.name || !botData.lowerPrice || !botData.upperPrice || !botData.amount) {
    return gbp_showMessage('❌ Fill all required fields', 'error')
  }

  gbp_activeBots.unshift(botData)
  gbp_showMessage(`✅ ${botData.name} created!`, 'success')
  gbp_renderBots()
  gbp_resetForm()
}

function gbp_resetForm() {
  const nameEl = gbp_$('gbp_botName') as HTMLInputElement
  if (nameEl) nameEl.value = ''
  const lp = gbp_$('gbp_lowerPrice'); if (lp) lp.value = ''
  const up = gbp_$('gbp_upperPrice'); if (up) up.value = ''
  const gr = gbp_$('gbp_grids');      if (gr) gr.value = '20'
  const am = gbp_$('gbp_amount');     if (am) am.value = ''
  gbp_updateDemoGrid()
}

function gbp_renderBots() {
  const list = gbp_$('gbp_botsList')
  if (!list) return
  if (gbp_activeBots.length === 0) {
    list.innerHTML = '<div style="text-align:center;padding:40px;color:rgba(255,255,255,0.5)">🤖 No active bots yet</div>'
    return
  }
  list.innerHTML = gbp_activeBots.map(bot => `
    <div class="bot-card">
      <strong>${bot.name}</strong><br>
      ${bot.symbol} | ${bot.exchange || 'coinbaseadvanced'}<br>
      Range: $${bot.lowerPrice.toLocaleString()} - $${bot.upperPrice.toLocaleString()}<br>
      Grids: ${bot.grids} | Amount: $${bot.amount}<br>
      Inc: B:${bot.incBuy}% S:${bot.incSell}% | Dev: B:${bot.devPriceBuy}% S:${bot.devPriceSell}%
    </div>`).join('')
}

function gbp_switchLangTab(lang: string, e: MouseEvent) {
  ;['js','cpp','zig'].forEach(l => {
    const el = gbp_$(`gbp_lang-${l}`)
    if (el) el.style.display = l === lang ? 'block' : 'none'
  })
  document.querySelectorAll('.lang-tab').forEach((btn: Element) => {
    const b = btn as HTMLButtonElement
    b.style.background = 'rgba(100,200,255,0.2)'
    b.style.color = '#64c8ff'
    b.style.borderColor = 'rgba(100,200,255,0.3)'
  })
  const target = e.target as HTMLButtonElement
  target.style.background = 'rgba(59,130,246,0.3)'
  target.style.color = '#3b82f6'
  target.style.borderColor = 'rgba(59,130,246,0.5)'
}

function gbp_showMessage(text: string, type = 'info') {
  const container = gbp_$('gbpMessages')
  if (!container) return
  const msg = document.createElement('div')
  msg.className = `message ${type}`
  msg.textContent = text
  container.appendChild(msg)
  setTimeout(() => msg.remove(), 4000)
}

onMounted(() => {
  gbp_updateLivePrices()
  gbp_priceTimer = setInterval(gbp_updateLivePrices, 2000)
  const nameEl = gbp_$('gbp_botName') as HTMLInputElement
  if (nameEl) nameEl.value = `gridBot_${Date.now().toString().slice(-5)}`
  gbp_renderBots()
  gbp_showMessage('✅ GridBot+ Full Complete loaded with DemoGridBot & QuickActionsPanel', 'success')
})

onUnmounted(() => {
  clearInterval(gbp_priceTimer)
})
</script>

<style scoped>
.gbp-root {
  font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Arial;
  background: #0f1419;
  color: #e0e0e0;
  font-size: 12px;
  padding: 16px;
  min-height: 100vh;
  position: relative;
}

.messages-container {
  position: fixed; top: 16px; right: 16px; z-index: 9999; max-width: 400px; pointer-events: none;
}
.message {
  padding: 12px 16px; border-radius: 6px; margin-bottom: 8px; font-size: 11px; font-weight: 600;
  animation: slideIn 0.3s ease;
}
@keyframes slideIn { from { transform: translateX(400px); opacity: 0; } to { transform: translateX(0); opacity: 1; } }
.message.info    { background: rgba(59,130,246,0.2); color: #3b82f6; border: 1px solid rgba(59,130,246,0.3); }
.message.success { background: rgba(16,235,4,0.2);   color: #10eb04; border: 1px solid rgba(16,235,4,0.3); }
.message.error   { background: rgba(255,77,79,0.2);  color: #ff4d4f; border: 1px solid rgba(255,77,79,0.3); }

.page-header {
  padding: 16px;
  background: linear-gradient(135deg, rgba(59,130,246,0.1), rgba(16,235,4,0.08));
  border-radius: 8px; border: 1px solid rgba(59,130,246,0.3); margin-bottom: 24px;
}
.page-title { font-size: 24px; font-weight: 800; color: #10eb04; }

.form-card {
  background: rgba(20,25,30,0.8); border: 2px solid rgba(59,130,246,0.3);
  border-radius: 8px; padding: 20px; margin-bottom: 24px;
}
.form-title {
  font-size: 16px; font-weight: 800; color: #3b82f6; margin-bottom: 16px;
  padding-bottom: 12px; border-bottom: 1px solid rgba(59,130,246,0.2);
}
.form-row { display: grid; grid-template-columns: repeat(auto-fit, minmax(150px, 1fr)); gap: 16px; margin-bottom: 16px; }
.form-group { display: flex; flex-direction: column; }
.form-group label { font-size: 10px; font-weight: 700; color: #3b82f6; margin-bottom: 6px; text-transform: uppercase; letter-spacing: 0.2px; }

input, select {
  padding: 10px; background: rgba(26,31,46,0.8); border: 1px solid rgba(59,130,246,0.3);
  border-radius: 6px; color: #e0e0e0; font-size: 11px; font-family: inherit;
}
input:focus, select:focus { outline: none; border-color: #3b82f6; box-shadow: 0 0 8px rgba(59,130,246,0.3); }

.price-buttons {
  display: grid; grid-template-columns: repeat(auto-fit, minmax(45px, 1fr));
  gap: 4px; margin-top: 8px; padding: 8px; background: rgba(0,0,0,0.2); border-radius: 4px;
}
.price-btn {
  padding: 4px 6px; font-size: 9px; font-weight: 600; border: 1px solid rgba(59,130,246,0.3);
  background: rgba(59,130,246,0.1); color: #3b82f6; border-radius: 3px; cursor: pointer; transition: all 0.2s;
}
.price-btn:hover { background: rgba(59,130,246,0.2); border-color: #3b82f6; transform: scale(1.05); }

.header-prices { display: grid; grid-template-columns: repeat(2, 1fr); gap: 12px; margin-bottom: 16px; }
.price-badge { padding: 10px; border-radius: 6px; text-align: center; font-weight: 700; font-size: 11px; }
.price-badge.bid { background: rgba(16,235,4,0.1); border: 1px solid rgba(16,235,4,0.3); color: #10eb04; }
.price-badge.ask { background: rgba(235,4,4,0.1);  border: 1px solid rgba(235,4,4,0.3);  color: #eb0404; }

.buy-sell-table { width: 100%; border-collapse: collapse; background: rgba(26,31,46,0.6); border-radius: 6px; overflow: hidden; margin-bottom: 16px; }
.buy-sell-table td { padding: 10px; border: 1px solid rgba(59,130,246,0.2); font-size: 10px; font-weight: 600; text-align: center; }
.buy-cell { background: rgba(16,235,4,0.05); }
.buy-cell.header-row { background: rgba(16,235,4,0.15); color: #10eb04; font-weight: 800; }
.sell-cell { background: rgba(235,4,4,0.05); }
.sell-cell.header-row { background: rgba(235,4,4,0.15); color: #eb0404; font-weight: 800; }
.buy-sell-table input { width: 100%; padding: 6px; font-size: 10px; }

.form-actions { display: flex; gap: 8px; margin-top: 20px; flex-wrap: wrap; }
.btn { padding: 10px 16px; border: none; border-radius: 6px; font-size: 11px; font-weight: 700; cursor: pointer; transition: all 0.3s; text-transform: uppercase; letter-spacing: 0.2px; }
.btn-primary { background: linear-gradient(135deg, #3b82f6, #10eb04); color: #0f1419; }
.btn-primary:hover { transform: translateY(-2px); box-shadow: 0 8px 16px rgba(59,130,246,0.4); }
.btn-demo  { background: rgba(250,204,21,0.2); color: #facc15; border: 1px solid rgba(250,204,21,0.3); }
.btn-demo:hover { background: rgba(250,204,21,0.3); }
.btn-lib   { background: rgba(100,200,255,0.2); color: #64c8ff; border: 1px solid rgba(100,200,255,0.3); }
.btn-lib:hover { background: rgba(100,200,255,0.3); }
.btn-reset { background: rgba(128,128,128,0.2); color: #888; border: none; }

.bots-section { background: rgba(20,25,30,0.8); border: 2px solid rgba(16,235,4,0.3); border-radius: 8px; padding: 20px; }
.bots-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(280px, 1fr)); gap: 12px; }

/* Quick Actions Panel */
.qa-panel {
  position: fixed; bottom: 20px; right: 20px;
  background: rgba(20,25,30,0.95); border: 2px solid rgba(250,204,21,0.3);
  border-radius: 8px; padding: 16px; max-width: 300px; z-index: 1000;
  cursor: move; box-shadow: 0 10px 40px rgba(0,0,0,0.5);
}
.qa-header {
  font-size: 12px; font-weight: 800; color: #facc15; margin-bottom: 12px;
  display: flex; justify-content: space-between; align-items: center; user-select: none;
}
.qa-buttons { display: grid; grid-template-columns: 1fr 1fr; gap: 8px; }
.qa-btn {
  padding: 8px; font-size: 10px; font-weight: 600; border: 1px solid rgba(59,130,246,0.3);
  background: rgba(59,130,246,0.1); color: #3b82f6; border-radius: 4px; cursor: pointer; transition: all 0.2s;
}
.qa-btn:hover { background: rgba(59,130,246,0.2); border-color: #3b82f6; }

/* Demo Grid Modal */
.demo-modal {
  position: fixed; top: 50%; left: 50%; transform: translate(-50%, -50%);
  background: rgba(20,25,30,0.95); border: 2px solid rgba(250,204,21,0.3);
  border-radius: 8px; padding: 20px; max-width: 800px; max-height: 600px;
  overflow-y: auto; z-index: 2000; cursor: move; box-shadow: 0 20px 50px rgba(0,0,0,0.7);
}
.demo-header {
  font-size: 14px; font-weight: 800; color: #facc15; margin-bottom: 16px;
  display: flex; justify-content: space-between; align-items: center; user-select: none;
}
.demo-stats { display: grid; grid-template-columns: repeat(auto-fit, minmax(120px, 1fr)); gap: 12px; margin-bottom: 16px; }
.stat-box { background: rgba(26,31,46,0.6); border: 1px solid rgba(59,130,246,0.2); border-radius: 6px; padding: 12px; text-align: center; }
.stat-label { font-size: 9px; color: #3b82f6; font-weight: 700; text-transform: uppercase; margin-bottom: 6px; }
.stat-value { font-size: 12px; font-weight: 700; color: #10eb04; }
.demo-orders { max-height: 400px; overflow-y: auto; }
.demo-order-row {
  display: grid; grid-template-columns: repeat(5, 1fr);
  gap: 8px; padding: 8px; background: rgba(26,31,46,0.4);
  border-radius: 4px; margin-bottom: 4px; font-size: 9px;
}
.demo-order-row.buy  { border-left: 3px solid #10eb04; }
.demo-order-row.sell { border-left: 3px solid #eb0404; }

/* Library Modal */
.library-modal {
  position: fixed; top: 50%; left: 50%; transform: translate(-50%, -50%);
  background: rgba(20,25,30,0.98); border: 2px solid rgba(100,200,255,0.3);
  border-radius: 8px; padding: 0; max-width: 1000px; max-height: 85vh;
  overflow: hidden; z-index: 2500; cursor: move;
  box-shadow: 0 20px 60px rgba(0,0,0,0.8); display: flex; flex-direction: column;
}
.library-header {
  font-size: 14px; font-weight: 800; color: #64c8ff; padding: 16px 20px;
  display: flex; justify-content: space-between; align-items: center;
  background: rgba(10,20,30,0.6); border-bottom: 1px solid rgba(100,200,255,0.2);
  flex-shrink: 0; user-select: none;
}
.library-content { overflow-y: auto; padding: 20px; flex: 1; }

.doc-section { margin-bottom: 28px; padding-bottom: 20px; border-bottom: 1px solid rgba(100,200,255,0.1); }
.doc-section:last-child { border-bottom: none; }
.doc-section h3 { font-size: 13px; color: #64c8ff; font-weight: 800; margin-bottom: 12px; text-transform: uppercase; letter-spacing: 0.5px; }
.doc-section p  { font-size: 11px; color: #b0b0b0; line-height: 1.6; margin-bottom: 8px; }
.doc-section ul, .doc-section ol { margin-left: 16px; margin-bottom: 8px; font-size: 11px; color: #b0b0b0; line-height: 1.6; }
.doc-section li { margin-bottom: 6px; }

.function-doc { background: rgba(26,31,46,0.4); border: 1px solid rgba(100,200,255,0.15); border-radius: 6px; padding: 14px; margin-bottom: 12px; }
.function-doc h4 { color: #10eb04; font-size: 12px; font-weight: 700; margin-bottom: 8px; font-family: 'Courier New', monospace; }
.function-doc p { font-size: 10px; margin-bottom: 6px; }
.function-doc p strong { color: #64c8ff; }

pre {
  background: rgba(0,0,0,0.3); padding: 12px; border-radius: 4px; overflow-x: auto;
  font-size: 10px; color: #10eb04; font-family: 'Courier New', monospace; line-height: 1.4;
  white-space: pre-wrap;
}

.lang-tab {
  background: rgba(100,200,255,0.2); color: #64c8ff; border: 1px solid rgba(100,200,255,0.3);
  padding: 8px 16px; border-radius: 4px; cursor: pointer; font-weight: 700; font-size: 11px;
  transition: all 0.2s;
}
.active-tab {
  background: rgba(59,130,246,0.3); color: #3b82f6; border-color: rgba(59,130,246,0.5);
}

.bot-card {
  background: rgba(26,31,46,0.6); border: 1px solid rgba(16,235,4,0.2);
  border-radius: 6px; padding: 12px; font-size: 10px; line-height: 1.7;
}

.x-btn { background: none; border: none; color: #facc15; cursor: pointer; font-size: 12px; padding: 0 4px; }

.hidden { display: none !important; }
</style>
