export default `
        <div class="intro">
          <div><div class="eyebrow">TRADING LAB / 001</div><h1>Build your grid<span>.</span></h1><p>One focused workspace for planning, testing, and tracking a grid strategy.</p></div>
          <div class="mode-card"><div class="mode-icon">◇</div><div><strong>PAPER MODE</strong><span>Manual price feed · no exchange orders</span></div></div>
        </div>

        <div id="notice" class="notice" role="status" aria-live="polite" hidden></div>

        <div class="main-grid">
          <section class="panel setup">
            <div class="panel-heading"><div><span class="section-n">01</span><h2>Configure bot</h2></div><span class="heading-sub">STRATEGY SETUP</span></div>
            <form id="grid-form">
              <div class="field-row"><label>Bot name<input id="name" value="Range study 01" maxlength="60" autocomplete="off"></label><label>Symbol<input id="symbol" value="DEMO/USDT" maxlength="32" autocomplete="off"></label></div>
              <div class="subheading">PRICE RANGE <span>01 / 03</span></div>
              <div class="field-row three"><label>Current price<input id="current-price" type="number" value="100" min="0" step="any"></label><label>Lower bound<input id="lower-price" type="number" value="90" min="0" step="any"></label><label>Upper bound<input id="upper-price" type="number" value="110" min="0" step="any"></label></div>
              <div class="field-row"><label>Grid intervals<input id="grid-count" type="number" value="8" min="2" max="200" step="1"></label><label>Spacing<select id="grid-type"><option value="linear">Linear</option><option value="geometric">Geometric</option></select></label></div>
              <div class="subheading">ORDER SIZE <span>02 / 03</span></div>
              <div class="field-row three"><label>Amount<input id="amount" type="number" value="100" min="0" step="any"></label><label>Currency<select id="amount-unit"><option value="quote">Quote</option><option value="base">Base</option></select></label><label>Allocation<select id="amount-type"><option value="per_grid">Per grid</option><option value="total">Total / intervals</option><option value="incremental">Incremental</option></select></label></div>
              <div class="field-row three"><label>Initial orders<select id="side"><option value="both">Buy + sell</option><option value="buy_only">Start buys only</option><option value="sell_only">Start sells only</option></select></label><label>Buy increment %<input id="increment-buy" type="number" value="0" min="0" max="100" step="any"></label><label>Sell increment %<input id="increment-sell" type="number" value="0" min="0" max="100" step="any"></label></div>
              <div class="subheading">PAPER WALLET <span>03 / 03</span></div>
              <div class="field-row three"><label>Starting base<input id="initial-base" type="number" value="5" min="0" step="any"></label><label>Starting quote<input id="initial-quote" type="number" value="1000" min="0" step="any"></label><label>Fee per fill %<input id="fee" type="number" value="0.1" min="0" max="5" step="any"></label></div>
              <p class="field-note">Buy orders reserve quote. Sell orders reserve base. Preview checks both before the bot starts.</p>
              <div class="form-actions"><button type="button" id="preview-btn" class="button secondary">Preview grid <span>↗</span></button><button type="submit" id="create-btn" class="button primary">Create paper bot <span>→</span></button></div>
            </form>
          </section>

          <section class="panel preview">
            <div class="panel-heading"><div><span class="section-n">02</span><h2>Grid preview</h2></div><span class="heading-sub">BEFORE YOU START</span></div>
            <div id="preview-empty" class="empty-preview"><div class="empty-art"><div class="bar b1"></div><div class="bar b2"></div><div class="bar b3"></div><div class="bar b4"></div><div class="bar b5"></div></div><strong>Your levels appear here</strong><p>Enter a range and preview the exact paper orders and funds needed.</p></div>
            <div id="preview-content" hidden>
              <div class="price-map"><div class="map-line"></div><div class="map-labels"><span id="map-low">—</span><span id="map-current">—</span><span id="map-high">—</span></div><div class="map-captions"><span>LOWER</span><span>CURRENT</span><span>UPPER</span></div></div>
              <div class="metrics"><div><span>BUY ORDERS</span><strong id="buy-count" class="buy">—</strong></div><div><span>SELL ORDERS</span><strong id="sell-count" class="sell">—</strong></div><div><span>QUOTE NEEDED</span><strong id="quote-needed">—</strong></div><div><span>BASE NEEDED</span><strong id="base-needed">—</strong></div></div>
              <div class="table-label">PLANNED LIMIT ORDERS <span id="order-total">—</span></div>
              <div class="order-scroll"><table><thead><tr><th>LEVEL</th><th>SIDE</th><th>PRICE</th><th>AMOUNT</th><th>NOTIONAL</th></tr></thead><tbody id="preview-orders"></tbody></table></div>
            </div>
          </section>
        </div>

        <div class="section-title"><div><span class="section-n">03</span><h2>Paper bots</h2></div><span id="bot-count">0 BOTS</span></div>
        <div class="bottom-grid"><section class="panel bot-list" id="bot-list"><div class="list-placeholder">No bots yet. Create one above to begin.</div></section><section class="panel bot-detail" id="bot-detail"><div class="list-placeholder">Select a bot to see its wallet, orders, and fills.</div></section></div>
        <footer>OMIBUS GRID · LOCAL PAPER TERMINAL <span>Simulation uses manual ticks and assumes limit fills at the level price.</span></footer>
`;
