const spawn = require('child_process').spawn;

process.env['PATH'] = process.env['PATH'] + ':' + process.env['LAMBDA_TASK_ROOT'];
exports.handler = (e, c, callback) => {
  'use strict';
  let process = spawn("rust-timetables");
  process.stdout.on('data', (data) => {
    console.log(`stdout: ${data}`);
  });
  
  process.stderr.on('data', (data) => {
    console.log(`stderr: ${data}`);
  });
  
  process.on('close', (code) => {
    console.log(`child process exited with code ${code}`);
    callback(null, 'ok');
  });
}
