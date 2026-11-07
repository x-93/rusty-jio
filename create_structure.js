const fs = require('fs');
const path = require('path');

const content = fs.readFileSync('prompt_tree.txt', 'utf8');
const lines = content.split(/\r?\n/);

let started = false;
const stack = [];
const createdDirs = [];
const createdFiles = [];

const invalidChars = /[<>:"|?*]/;
const badNames = [];

for (let i = 0; i < lines.length; i++) {
  const line = lines[i];
  if (line.includes('└── rusty-jio/')) {
    started = true;
    continue;
  }
  if (!started) continue;
  if (line.trim().startsWith('</USER_REQUEST>')) break;
  if (!line.trim()) continue;

  const match = line.match(/^(.*?)((?:├──|└──)\s*)(.*)$/);
  if (!match) continue;
  const prefix = match[1];
  const depth = prefix.length / 4;
  const rawName = match[3].trim();
  const isDir = rawName.endsWith('/');
  const name = isDir ? rawName.slice(0, -1) : rawName;

  if (invalidChars.test(name)) {
    badNames.push({ line: i + 1, name });
  }

  while (stack.length >= depth) {
    stack.pop();
  }
  const parentPath = stack.map(s => s.name).join('/');
  const relPath = parentPath ? parentPath + '/' + name : name;
  stack.push({ depth, name, isDir });

  const targetPath = path.resolve(__dirname, relPath);

  if (isDir) {
    fs.mkdirSync(targetPath, { recursive: true });
    createdDirs.push(relPath);
  } else {
    const dir = path.dirname(targetPath);
    fs.mkdirSync(dir, { recursive: true });
    if (!fs.existsSync(targetPath)) {
      fs.writeFileSync(targetPath, '');
    }
    createdFiles.push(relPath);
  }
}

console.log('Bad names count:', badNames.length);
if (badNames.length > 0) {
  console.log('Bad names:', badNames);
}
console.log('Total dirs created/ensured:', createdDirs.length);
console.log('Total files created/ensured:', createdFiles.length);

// Verify all files and dirs exist
let missingCount = 0;
for (const d of createdDirs) {
  const p = path.resolve(__dirname, d);
  if (!fs.existsSync(p) || !fs.statSync(p).isDirectory()) {
    console.error('Missing dir:', d);
    missingCount++;
  }
}
for (const f of createdFiles) {
  const p = path.resolve(__dirname, f);
  if (!fs.existsSync(p) || !fs.statSync(p).isFile()) {
    console.error('Missing file:', f);
    missingCount++;
  }
}
console.log('Missing items count:', missingCount);

