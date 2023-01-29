# how-active
`how-active` is a terminal UI-based tool for analysing a user's activity on Discord.
Input a USER ID (enable Developer's mode in Discord, right-click on a user and select `Copy ID`)
and a GUILD or CHANNEL ID (obtained the same way, except by right-clicking on a server or channel).

After verifying your target user and target channel/guild in the top right window, 
you may hit the `s` key to start the procedure. `how-active` will use the Discord 
search API to grab the messages sent by the given user and will parse the timestamp 
to increment that given hour in the chart.

## Getting your authorization token
You may use tools like [Discord-Token-Dumper by Sorrow446](https://github.com/Sorrow446/Discord-Token-Dumper) 
or do it manually by following instructions in the next two subsections.

### Desktop App
1. Add `"DANGEROUS_ENABLE_DEVTOOLS_ONLY_ENABLE_IF_YOU_KNOW_WHAT_YOURE_DOING": true` to your Discord `settings.json`.
2. Proceed by hitting `Ctrl+Shift+i` to open up the Developer Console.
3. Navigate to the `Application` tab.
4. Filter by `token`.
5. Copy the `value` corresponding to the `key` of `token`.
6. This is your token, write it to an environment variable or supply it as an argument.

### Browser App
1. Pull up the Developer Console by hitting `Ctrl+Shift+i` or `F12`.
2. On Chrome navigate to the `Application` tab, on Firefox navigate to the `Storage` tab.
3. Go to `Local Storage`.
4. Filter by `token`.
5. Copy the `value` of `token`.
6. You have your token.

## Compiling from source
```bash
git clone https://github.com/phoreverpheebs/how-active
cd how-active
cargo build
```

# Disclaimer
The Discord TOS does say not to automate your account; whether what this program does
counts as automating or not, I will not be liable for any banned accounts.
