const fs=require('fs');
const vm=require('vm');
const ctx={};
vm.createContext(ctx);
for (const f of ['dic.js','lang.js','init.js']) vm.runInContext(fs.readFileSync(f,'utf8'),ctx);
fs.writeFileSync('l.json',JSON.stringify(ctx.l));
fs.writeFileSync('d.json',JSON.stringify(ctx.d));
fs.writeFileSync('defaults.json',JSON.stringify(ctx.Mopi2));
function shape(o,depth){ if(typeof o!=='object'||o===null)return typeof o==='string'?'s':o; if(depth==0)return '{..}'; const r={};for(const k in o){r[k]=shape(o[k],depth-1)}return r;}
console.log(JSON.stringify(shape(ctx.l,2)));
