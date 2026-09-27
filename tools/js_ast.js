// Dump ESTree ASTs of JavaScript files as JSON lines: {"file": ..., "ast": ...}.
// Usage: node tools/js_ast.js ACORN_DIR ROOT_DIR > out.jsonl
const path = require('path');
const fs = require('fs');
const acorn = require(path.resolve(process.argv[2], 'acorn'));
const root = process.argv[3];
function walk(dir) {
  for (const name of fs.readdirSync(dir).sort()) {
    const p = path.join(dir, name);
    if (fs.statSync(p).isDirectory()) { if (name !== 'node_modules' && name !== 'test') walk(p); continue; }
    if (!name.endsWith('.js') || name.includes('test') || name.includes('spec')) continue;
    try {
      const ast = acorn.parse(fs.readFileSync(p, 'utf8'), { ecmaVersion: 'latest', sourceType: 'script', allowReturnOutsideFunction: true, allowHashBang: true });
      process.stdout.write(JSON.stringify({ file: path.relative(root, p), ast }) + '\n');
    } catch (e) { process.stderr.write(`skip ${p}: ${e.message}\n`); }
  }
}
walk(root);
