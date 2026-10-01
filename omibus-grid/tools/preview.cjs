// Local UI preview only: expose UI assets and feature configuration, not the repository.
const http = require('node:http');
const fs = require('node:fs');
const path = require('node:path');
const { spawn } = require('node:child_process');
const root = path.resolve(__dirname, '..');
const ui = path.join(root, 'ui');
// Optional visual-test fixture; never part of the shipped desktop app.
const reportFlag = process.argv.indexOf('--ai-report');
const report = reportFlag >= 0 ? JSON.parse(fs.readFileSync(path.resolve(process.argv[reportFlag + 1]), 'utf8').replace(/^\uFEFF/, '')) : null;
const liveChart = process.argv.includes('--live-chart');
let chartBusy = false;
http.createServer((request, response) => {
  const pathname = new URL(request.url, 'http://127.0.0.1').pathname;
  if (pathname === '/chart-preview' && liveChart && request.method === 'POST') {
    response.setHeader('Content-Type','application/json');
    if (request.headers.origin && request.headers.origin !== 'http://127.0.0.1:4174') { response.writeHead(403).end('{}'); return; }
    if (chartBusy) { response.end(JSON.stringify({error:'A chart request is in progress',retry_seconds:5})); return; }
    let body='';
    request.on('data',chunk=>{body+=chunk;if(body.length>16000)request.destroy();});
    request.on('end',()=>{
      let config;try{config=JSON.parse(body);}catch{response.writeHead(400).end('{}');return;}
      chartBusy=true;
      const child=spawn(path.join(root,'ai-worker/runtime/python.exe'),['-I',path.join(root,'ai-worker/chart_worker.py')],{windowsHide:true,stdio:['pipe','pipe','pipe']});
      let output='',error='';const timeout=setTimeout(()=>child.kill(),80000);
      child.stdout.on('data',chunk=>{output+=chunk;if(output.length>3000000)child.kill();});
      child.stderr.on('data',chunk=>{error+=chunk;});
      child.on('error',e=>{chartBusy=false;clearTimeout(timeout);if(!response.writableEnded)response.end(JSON.stringify({error:e.message,retry_seconds:15}));});
      child.on('close',code=>{chartBusy=false;clearTimeout(timeout);if(!response.writableEnded)response.end(code===0?output:JSON.stringify({error:error.slice(0,1000)||'Chart reader failed',retry_seconds:15}));});
      response.on('close',()=>{if(!response.writableEnded)child.kill();});
      child.stdin.end(JSON.stringify(config));
    });
    return;
  }
  if (pathname === '/preview-bridge.js' && report) {
    const status = { running: false, resume: false, config: report.config, error: null,
      worker: { phase: 'waiting', report }, data_directory: 'Browser preview of a saved research report',
      catalog: JSON.parse(fs.readFileSync(path.join(root,'ai-worker/catalog.json'),'utf8')) };
    response.setHeader('Content-Type', 'text/javascript');
    response.end(`window.__TAURI__={core:{invoke:async(name,args)=>{if(name==='get_modules')return {grid:true,charts:true,ai:true};if(name==='ai_status')return ${JSON.stringify(status)};if(name==='ai_chart_snapshot'&&${liveChart})return await(await fetch('/chart-preview',{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify(args.config)})).json();throw new Error('Browser preview only. Open the desktop app for worker controls.');}}};document.addEventListener('DOMContentLoaded',()=>{const note=document.createElement('div');note.textContent='DEVELOPMENT PREVIEW · ${liveChart?'Live chart':'Saved data'} · Training controls work in the desktop executable';note.setAttribute('role','note');document.body.prepend(note);});`);
    return;
  }
  const file = pathname === '/modules.json' ? path.join(root, 'modules.json') :
    path.resolve(root, '.' + (pathname === '/ui/' ? '/ui/index.html' : pathname));
  if (file !== path.join(root, 'modules.json') && !file.startsWith(ui + path.sep)) {
    response.writeHead(403).end(); return;
  }
  fs.readFile(file, (error, content) => {
    if (error) { response.writeHead(404).end(); return; }
    response.setHeader('Cache-Control', 'no-store');
    const config = JSON.parse(fs.readFileSync(path.join(root, 'src-tauri/tauri.conf.json'), 'utf8'));
    response.setHeader('Content-Security-Policy', config.app.security.csp);
    response.setHeader('Content-Type', ({ '.html': 'text/html', '.js': 'text/javascript', '.css': 'text/css', '.json': 'application/json' })[path.extname(file)] || 'application/octet-stream');
    if (report && file === path.join(ui, 'index.html')) content = content.toString().replace('<head>', '<head><script src="/preview-bridge.js"></script>');
    response.end(content);
  });
}).listen(4174, '127.0.0.1', () => console.log('UI preview: http://127.0.0.1:4174/ui/'));
