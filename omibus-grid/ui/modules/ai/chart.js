// Local live research chart. Market snapshots come from a read-only native command.
export function createLiveChart(root) {
  const ns='http://www.w3.org/2000/svg';
  let snapshot,report,visible=120,offset=0;
  const colors=['#b9f05a','#69b9ff','#ffb768','#d39bff','#54ddc3','#ee91b7'];
  const format=n=>Number.isFinite(n) ? n.toLocaleString('en-US',{maximumFractionDigits:4}) : '—';
  const clock=t=>new Date(t).toISOString().slice(5,16).replace('T',' ')+' UTC';
  const controls=document.createElement('div'); controls.className='ai-chart-tools';
  const body=document.createElement('div'); body.className='ai-live-canvas';
  const hover=document.createElement('p'); hover.className='ai-chart-readout';
  const choices={formations:true,rules:true,trades:true};
  function button(label,action) { const b=document.createElement('button'); b.type='button'; b.textContent=label; b.addEventListener('click',action); controls.append(b); }
  button('Zoom +',()=>{visible=Math.max(40,Math.round(visible/1.5));render();});
  button('Zoom −',()=>{visible=Math.min(500,Math.round(visible*1.5));render();});
  button('← Earlier',()=>{offset=Math.min(Math.max(0,(snapshot?.candles.length||600)-100-visible),offset+Math.floor(visible/2));render();});
  button('Later →',()=>{offset=Math.max(0,offset-Math.floor(visible/2));render();});
  button('Live',()=>{offset=0;render();});
  for(const [key,label] of [['formations','Formations'],['rules','Rule triggers'],['trades','BUY / SELL / TP']]) {
    const holder=document.createElement('label'),input=document.createElement('input');input.type='checkbox';input.checked=true;
    input.addEventListener('change',()=>{choices[key]=input.checked;render();});holder.append(input,document.createTextNode(label));controls.append(holder);
  }
  root.replaceChildren(controls,hover,body);
  function svg(height,label) {
    const element=document.createElementNS(ns,'svg');element.setAttribute('viewBox',`0 0 1200 ${height}`);element.setAttribute('role','img');element.setAttribute('aria-label',label);return element;
  }
  function add(parent,tag,attrs={},text) {
    const e=document.createElementNS(ns,tag);for(const [k,v] of Object.entries(attrs))e.setAttribute(k,String(v));
    if(text!==undefined)e.textContent=text;parent.append(e);return e;
  }
  function render() {
    if(!snapshot?.candles.length)return;
    body.replaceChildren();
    const all=snapshot.candles,end=Math.max(101,all.length-offset),start=Math.max(100,end-visible),data=all.slice(start,end);
    if(!data.length)return;
    const first=data[0][0],last=data.at(-1)[0],step=all[1][0]-all[0][0];
    const x=time=>24+(time-first)/Math.max(step,last-first)*1088;
    const chart=svg(390,`Live ${snapshot.config.symbol} ${snapshot.config.interval} candlestick chart with indicators, formations and triggers`);
    const overlays=snapshot.panels.filter(p=>p.overlay),oscillators=snapshot.panels.filter(p=>!p.overlay);
    const bounds=data.flatMap(c=>[c[2],c[3]]);
    for(const panel of overlays)for(const s of panel.series)bounds.push(...s.values.slice(start,end).filter(Number.isFinite));
    let low=Math.min(...bounds),high=Math.max(...bounds);const padding=(high-low||high*.01)*.12;low-=padding;high+=padding;
    const y=value=>330-(value-low)/(high-low)*290;
    const defs=add(chart,'defs');const clip=add(defs,'clipPath',{id:'ai-live-price-clip'});add(clip,'rect',{x:20,y:22,width:1100,height:315});
    const plot=add(chart,'g',{'clip-path':'url(#ai-live-price-clip)'});
    for(let i=0;i<5;i++){const p=low+(high-low)*i/4;add(chart,'line',{x1:20,x2:1115,y1:y(p),y2:y(p),stroke:'#28343d'});add(chart,'text',{x:1122,y:y(p)+4,fill:'#9daebb','font-size':11},format(p));}
    const width=Math.max(1.5,Math.min(11,800/data.length));
    for(const c of data){const color=c[4]>=c[1]?'#65d4a0':'#ef8585';add(plot,'line',{x1:x(c[0]),x2:x(c[0]),y1:y(c[2]),y2:y(c[3]),stroke:color});add(plot,'rect',{x:x(c[0])-width/2,y:Math.min(y(c[1]),y(c[4])),width,height:Math.max(1,Math.abs(y(c[1])-y(c[4]))),fill:color,opacity:(snapshot.provisional && c===all.at(-1)) ? 0.6 : 1});}
    function line(parent,series,scale,color){let points=[];for(let i=start;i<end;i++){const v=series.values[i];if(Number.isFinite(v)){points.push(`${x(all[i][0])},${scale(v)}`);}else if(points.length){add(parent,'polyline',{points:points.join(' '),fill:'none',stroke:color,'stroke-width':1.6});points=[];}}if(points.length)add(parent,'polyline',{points:points.join(' '),fill:'none',stroke:color,'stroke-width':1.6});}
    let colorIndex=0;const legend=[];
    for(const p of overlays)for(const s of p.series){const color=colors[colorIndex++%colors.length];line(plot,s,y,color);legend.push([s.name,color]);}
    const formations=choices.formations ? [...snapshot.confirmed.filter(p=>p.confirmed_at>=first&&p.confirmed_at<last+step).slice(-6),...snapshot.forming.slice(-6)] : [];
    for(const pattern of formations){
      const provisional=pattern.status==='forming',color=provisional?'#eabb69':pattern.direction==='bullish'?'#81dabe':'#afaeff';
      add(plot,'polyline',{points:pattern.points.map(p=>`${x(p.time)},${y(p.price)}`).join(' '),fill:'none',stroke:color,'stroke-width':2,'stroke-dasharray':provisional?'5 5':'none'});
      for(const boundary of pattern.lines||[]) {const [a,b]=boundary.points;add(plot,'line',{x1:x(a.time),x2:x(b.time),y1:y(a.price),y2:y(b.price),stroke:color,'stroke-dasharray':'5 4','stroke-width':1.4});}
      const anchor=pattern.points.at(-1);
      add(plot,'text',{x:x(anchor.time),y:Math.max(38,Math.min(320,y(anchor.price)-14)),fill:color,'font-size':11,'text-anchor':'middle'},pattern.name+(provisional?' · forming':''));
      if(!provisional) marker(pattern.confirmed_at,pattern.close,pattern.direction==='bullish'?'BREAK ↑':'BREAK ↓',color);
    }
    function marker(time,price,label,color) {
      const candle=data.find(c=>c[0]<=time&&c[6]>=time);if(!candle||!Number.isFinite(price))return;
      const cx=x(candle[0]),cy=y(price);add(plot,'circle',{cx,cy,r:4,fill:color,stroke:'#0c1218'});
      const above=/SELL|TP|EXIT|↓/.test(label);
      add(plot,'text',{x:cx,y:Math.max(36,Math.min(323,cy+(above?-12:20))),fill:color,'text-anchor':'middle','font-size':11,'font-weight':'bold'},label);
    }
    if(choices.rules)for(const m of snapshot.custom_triggers)marker(m.time,m.price,m.kind,'#ffc875');
    if(choices.trades&&report){
      for(const trade of report.trades){if(trade.entry_price!==undefined)marker(trade.entry_time,trade.entry_price,trade.side==='LONG'?'BUY':'SELL',trade.side==='LONG'?'#81edac':'#ff9898');if(trade.status==='CLOSED')marker(trade.exit_time,trade.exit_price,trade.reason,trade.reason==='TP'?'#66d8ff':'#edb66e');}
      if(report.position?.target_price){const p=report.position.target_price;add(plot,'line',{x1:24,x2:1112,y1:y(p),y2:y(p),stroke:'#66d8ff','stroke-dasharray':'7 5'});add(plot,'text',{x:35,y:y(p)-5,fill:'#66d8ff','font-size':12},`TP ${report.position.side} ${format(p)}`);}
    }
    for(let i=0;i<5;i++){const c=data[Math.floor((data.length-1)*i/4)];add(chart,'text',{x:x(c[0]),y:367,fill:'#9daebb','font-size':10,'text-anchor':i===0?'start':i===4?'end':'middle'},clock(c[0]));}
    const lastBar=all.at(-1);
    hover.textContent=`${snapshot.config.symbol} · ${snapshot.config.interval} · Last ${format(lastBar[4])} · ${offset?'Historical view':'Latest candles'} · ${snapshot.provisional?'Current candle provisional':'All shown candles closed'}`;
    const cross=add(chart,'line',{y1:25,y2:335,x1:0,x2:0,stroke:'#cad5de','stroke-dasharray':'2 4',visibility:'hidden'});
    chart.addEventListener('pointermove',event=>{const rect=chart.getBoundingClientRect();const coordinate=(event.clientX-rect.left)/rect.width*1200;const index=Math.max(0,Math.min(data.length-1,Math.round((coordinate-24)/1088*(data.length-1))));const c=data[index];cross.setAttribute('x1',x(c[0]));cross.setAttribute('x2',x(c[0]));cross.setAttribute('visibility','visible');hover.textContent=`${clock(c[0])} · O ${format(c[1])} H ${format(c[2])} L ${format(c[3])} C ${format(c[4])} · Volume ${format(c[5])}`;});
    chart.addEventListener('pointerleave',()=>cross.setAttribute('visibility','hidden'));
    body.append(chart);
    if(legend.length){const labels=document.createElement('div');labels.className='ai-chart-legend';for(const [name,color]of legend){const label=document.createElement('span');label.textContent=name;label.style.color=color;labels.append(label);}body.append(labels);}
    for(const [panelIndex,panel]of oscillators.entries()){
      const pane=svg(150,panel.label+' indicator panel');const values=panel.series.flatMap(s=>s.values.slice(start,end).filter(Number.isFinite));
      const rules=[...snapshot.config.triggers.buy,...snapshot.config.triggers.sell].filter(r=>r.right==='number'&&panel.series.some(s=>s.name===r.left));
      values.push(...rules.map(r=>r.value));if(!values.length)continue;
      let min=Math.min(...values),max=Math.max(...values);
      if(['rsi','stochastic','mfi','aroon','adx'].includes(panel.id)){min=Math.min(min,0);max=Math.max(max,100);}
      const span=max-min||1;min-=span*.1;max+=span*.1;const py=value=>115-(value-min)/(max-min)*80;
      add(pane,'text',{x:24,y:19,fill:'#c8d8e3','font-size':12},panel.label);
      for(let i=0;i<3;i++){const p=min+(max-min)*i/2;add(pane,'line',{x1:20,x2:1115,y1:py(p),y2:py(p),stroke:'#26323a'});add(pane,'text',{x:1122,y:py(p)+3,fill:'#92a3b1','font-size':10},format(p));}
      for(const [i,s]of panel.series.entries()){const color=colors[(panelIndex+i)%colors.length];line(pane,s,py,color);add(pane,'text',{x:230+i*220,y:19,fill:color,'font-size':11},s.name+' '+format(s.values[end-1]));}
      for(const rule of rules){add(pane,'line',{x1:24,x2:1112,y1:py(rule.value),y2:py(rule.value),stroke:'#ffc875','stroke-dasharray':'5 4'});add(pane,'text',{x:35,y:py(rule.value)-4,fill:'#ffc875','font-size':10},'Rule level '+rule.value);}
      body.append(pane);
    }
  }
  return {update(next,modelReport){snapshot=next;report=modelReport;render();},dispose(){root.replaceChildren();}};
}
