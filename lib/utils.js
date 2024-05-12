'use strict';

exports.generateAppleScript = dir => {
  const terminalCommand = `tell application "Terminal"
    do script "cd ${dir}"  in front window
  end tell`.split('\n').map(line => (` -e '${line.trim()}'`)).join('');

  const iTermCommand = `tell application "iTerm"
    tell current session of current window
      write text "cd ${dir}"
    end tell
  end tell`.split('\n').map(line => (` -e '${line.trim()}'`)).join('');

  const warpCommand = `tell application "Warp"
    tell current session of current window
      write text "cd ${dir}"
    end tell
  end tell`.split('\n').map(line => (` -e '${line.trim()}'`)).join('');

  const currentApp = `tell application "System Events"
    set activeApp to name of first application process whose frontmost is true
  end tell`.split('\n').map(line => (` -e '${line.trim()}'`)).join('');

  if (process.env.TERM_PROGRAM === 'Warp') {
    return `osascript ${warpCommand}`;
  } else {
    return `[ \`osascript ${currentApp}\` = "Terminal" ] && osascript ${terminalCommand} >/dev/null || osascript ${iTermCommand}`;
  }
};

exports.generateWarpScript = dir => {
  return `tell application "Warp"
    tell current session of current window
      write text "cd ${dir}"
    end tell
  end tell`.split('\n').map(line => (` -e '${line.trim()}'`)).join('');
};
