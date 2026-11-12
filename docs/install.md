# Install

`diffmate watch` runs as a GUI launchd agent. The live plist stays in
dotfiles at `~/Library/LaunchAgents/dev.diffmate.watch.plist`. This
repository does not commit that plist or a README.

## Binary

`ProgramArguments` is the absolute path of `~/.local/bin/diffmate` and
then the argument `watch`. launchd does not expand a tilde. A tilde in
`ProgramArguments` was measured as exit 78.

```text
mise run build
cp target/release/diffmate ~/.local/bin/diffmate
```

## Log directory

Before the first bootstrap, create the log directory. If that directory
already exists, `chmod 0700` it. launchd creates a missing parent mode
0755 or 0744 and the log files mode 0644 even when `Umask` is the string
0077.

```text
mkdir -p -m 0700 ~/Library/Logs/diffmate
chmod 0700 ~/Library/Logs/diffmate
```

## Agent

Replace every `/Users/YOU` with the output of `echo $HOME`. On this
machine that home is `/Users/apaderin`. `PATH` is the directories from
`which git`, `which gh`, and `which grok`, duplicates removed, then
`/usr/bin:/bin:/usr/sbin:/sbin`. Do not copy the full shell `PATH`. Do
not put a token in the plist. The machine `PATH` written with an
absolute home is
`/opt/homebrew/bin:/Users/apaderin/.grok/bin:/Users/apaderin/.local/bin:/usr/bin:/bin:/usr/sbin:/sbin`.
A plist integer umask is decimal, so the example uses the string 0077.
The integer form was measured as umask 0115.

The keys are `Label` `dev.diffmate.watch`, `ProgramArguments`,
`EnvironmentVariables` with `PATH` only, `KeepAlive` true, `RunAtLoad`
true, `ThrottleInterval` integer 10, `Umask` the string 0077,
`StandardOutPath`, and `StandardErrorPath`. Do not add
`WorkingDirectory`, `ProcessType`, or any other key.

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>Label</key>
  <string>dev.diffmate.watch</string>
  <key>ProgramArguments</key>
  <array>
    <string>/Users/YOU/.local/bin/diffmate</string>
    <string>watch</string>
  </array>
  <key>EnvironmentVariables</key>
  <dict>
    <key>PATH</key>
    <string>/opt/homebrew/bin:/Users/YOU/.grok/bin:/Users/YOU/.local/bin:/usr/bin:/bin:/usr/sbin:/sbin</string>
  </dict>
  <key>KeepAlive</key>
  <true/>
  <key>RunAtLoad</key>
  <true/>
  <key>ThrottleInterval</key>
  <integer>10</integer>
  <key>Umask</key>
  <string>0077</string>
  <key>StandardOutPath</key>
  <string>/Users/YOU/Library/Logs/diffmate/watch.out.log</string>
  <key>StandardErrorPath</key>
  <string>/Users/YOU/Library/Logs/diffmate/watch.log</string>
</dict>
</plist>
```

The shell expands `$UID`. `$UID` must not appear inside the plist.

## Install

```text
launchctl bootstrap gui/$UID ~/Library/LaunchAgents/dev.diffmate.watch.plist
```

## After login

The reboot check is manual. This phase does not reboot the machine.
After login, `diffmate doctor` prints `runner lock: ok — held by watch`
and exits 0. `watch` already waits for the runner lock instead of
exiting, and already exits 0 on SIGTERM, which is what `KeepAlive`
needs. The agent runs in the GUI session, not at the login window.

## Update

A second bootstrap is not how a new binary is picked up. Copy the binary
again, then kickstart the job that is already loaded:

```text
cp target/release/diffmate ~/.local/bin/diffmate
launchctl kickstart -k gui/$UID/dev.diffmate.watch
```

## Remove

```text
launchctl bootout gui/$UID/dev.diffmate.watch
```

## Empty the logs

`StandardErrorPath` is `~/Library/Logs/diffmate/watch.log`.
`StandardOutPath` is `~/Library/Logs/diffmate/watch.out.log`. launchd
appends both streams. `diffmate` does not open a log. `watch` prints one
timestamped line per event to stderr, and launchd writes that stream to
`watch.log`.

Both files grow. There is no rotator in `diffmate` and no `newsyslog`
example. To empty them: bootout, then truncate or delete the two files,
then bootstrap. If the directory was removed, run
`mkdir -p -m 0700 ~/Library/Logs/diffmate` before bootstrap again. An
existing directory is not mode-changed by `mkdir -p`, which is why
`chmod 0700` stays on the page when the directory already exists.
