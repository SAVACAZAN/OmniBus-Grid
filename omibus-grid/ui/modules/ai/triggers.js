export function createTriggerEditor(root, getSeries, onChange) {
  let rules={buy:[],sell:[]};
  function element(tag,text) {const e=document.createElement(tag);if(text!==undefined)e.textContent=text;return e;}
  function render(message='') {
    root.replaceChildren();const available=getSeries();
    root.append(element('p','All conditions on a side must pass AND its learned score must meet the threshold. Conditions use completed candles. No conditions means model score only.'));
    if(message)root.append(element('p',message));
    for(const side of ['buy','sell']) {
      const group=element('div');group.className='ai-rule-group';group.append(element('h3',side==='buy'?'BUY · open LONG':'SELL · open SHORT'));
      for(const [index,rule] of rules[side].entries()) {
        const row=element('div');row.className='ai-rule';
        const select=(field,options,label)=>{const input=element('select');input.setAttribute('aria-label',`${side} condition ${index+1} ${label}`);for(const [value,text]of options){const o=element('option',text);o.value=value;input.append(o);}input.value=rule[field];input.addEventListener('change',()=>{rule[field]=input.value;if(field==='right')render();onChange();});row.append(input);return input;};
        select('left',available.map(s=>[s,s]),'indicator');
        select('op',[['above','is above'],['below','is below'],['cross_above','crosses above'],['cross_below','crosses below']],'comparison');
        select('right',[['number','A number'],...available.map(s=>[s,s])],'compare with');
        const value=element('input');value.type='number';value.step='any';value.min='-1000000000000';value.max='1000000000000';value.value=rule.value;value.required=true;value.disabled=rule.right!=='number';value.setAttribute('aria-label',`${side} condition ${index+1} level`);value.addEventListener('change',()=>{rule.value=Number(value.value);onChange();});row.append(value);
        const remove=element('button','Remove');remove.type='button';remove.addEventListener('click',()=>{rules[side].splice(index,1);render();onChange();});row.append(remove);group.append(row);
      }
      const add=element('button',`+ ${side.toUpperCase()} condition`);add.type='button';add.disabled=!available.length||rules[side].length>=4;add.addEventListener('click',()=>{rules[side].push({left:available[0],op:side==='buy'?'cross_above':'cross_below',right:'number',value:available[0]==='rsi'?(side==='buy'?30:70):0});render();onChange();});group.append(add);root.append(group);
    }
  }
  return {
    set(value){rules=JSON.parse(JSON.stringify(value||{buy:[],sell:[]}));render();},
    get(){return Object.fromEntries(['buy','sell'].map(side=>[side,rules[side].map(rule=>({...rule,value:rule.right==='number'?Number(rule.value):0}))]));},
    refresh(){const available=getSeries();let removed=0;for(const side of ['buy','sell'])rules[side]=rules[side].filter(rule=>{const valid=available.includes(rule.left)&&(rule.right==='number'||available.includes(rule.right));if(!valid)removed++;return valid;});render(removed?`${removed} condition(s) removed because their indicator was unchecked.`:'');},
  };
}
