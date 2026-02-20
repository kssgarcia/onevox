# macOS Accessibility Permissions Setup for Vox

## Why This Is Needed

Global hotkeys and text injection require **Accessibility** permissions on macOS. Without these, the OS will block:

- Monitoring keyboard shortcuts system-wide
- Typing text into other applications

---

## Step-by-Step Guide

### Method 1: Automatic Prompt (Easiest)

1. **Run the daemon**:

   ```bash
   ./target/release/vox daemon
   ```

2. **Press your hotkey** (e.g., `Cmd+Shift+Space`)

3. **macOS should show a popup** saying:

   ```
   "Terminal" would like to control this computer using accessibility features.
   ```

4. Click **"Open System Settings"**

5. Your Terminal app should appear in the list - **toggle it ON**

6. **Restart the daemon**:

   ```bash
   # Press Ctrl+C to stop the daemon
   ./target/release/vox daemon
   ```

---

### Method 2: Manual Setup (If No Prompt Appears)

#### Step 1: Open System Settings

Click the Apple menu → **System Settings**

#### Step 2: Go to Privacy & Security

1. In the left sidebar, scroll down to **Privacy & Security**
2. Click it

#### Step 3: Find Accessibility

1. Scroll down in the main panel
2. Look for **Accessibility** (it's alphabetically ordered)
3. Click **Accessibility**

#### Step 4: Unlock to Make Changes

1. You'll see a lock icon 🔒 at the bottom
2. Click the lock
3. Enter your Mac password
4. The lock should turn to 🔓 (unlocked)

#### Step 5: Add Your Terminal App

**Option A - If Terminal is already listed:**

1. Find **Terminal** (or **iTerm**, or **Warp**, or whatever you're using)
2. Make sure the toggle switch is **ON** (blue)
3. If it's already ON, try:
   - Turn it **OFF**
   - Turn it **ON** again
   - This refreshes the permission

**Option B - If Terminal is NOT listed:**

1. Click the **"+" button** (bottom left)
2. Navigate to:
   - **Applications** → **Utilities** → **Terminal.app**
   - OR if using iTerm: **Applications** → **iTerm.app**
3. Select it and click **Open**
4. The app should now appear in the list with the toggle **ON**

#### Step 6: Lock Settings Again

Click the lock icon 🔓 again to prevent accidental changes

---

## How to Know Which Terminal App You're Using

Run this command to see your current shell's parent process:

```bash
ps -p $$ -o comm=
```

**Output tells you**:

- `-bash` or `/bin/zsh` → You're using **Terminal.app**
- `iTerm2` → You're using **iTerm**
- `warp` → You're using **Warp**
- `alacritty` → You're using **Alacritty**

---

## Verify Permissions Are Working

After granting permissions:

1. **Restart the daemon**:

   ```bash
   ./target/release/vox daemon
   ```

2. **Look for these logs**:

   ```
   ✅ Hotkey registered: Cmd+Shift+Space
   ✅ Hotkey listener started
   Hotkey listener thread started, monitoring for events...
   Dictation engine event loop started
   ```

3. **Press the hotkey** (`Cmd+Shift+Space`)

4. **You should see**:

   ```
   GlobalHotKeyEvent received: id=..., state=Pressed
   Hotkey matched! Sending event: Pressed
   🎹 Hotkey pressed - starting dictation
   🎤 Starting dictation
   ```

---

## Troubleshooting

### Still No Logs When Pressing Hotkey?

**Try a different hotkey** to rule out conflicts:

```bash
# Create/edit config
nano ~/.config/vox/config.toml

# Change the trigger line to:
trigger = "Cmd+Shift+F12"

# Save (Ctrl+O, Enter, Ctrl+X)

# Restart daemon
./target/release/vox daemon
```

Then press `Cmd+Shift+F12`

### "Terminal" Doesn't Appear in Accessibility List

Some terminals need to be added manually:

1. Click the **"+"** button
2. Press `Cmd+Shift+G` (Go to folder)
3. Type: `/Applications/Utilities/Terminal.app`
4. Press Enter
5. Select Terminal.app
6. Click Open

### macOS Blocks the Permission

On newer macOS versions (Ventura/Sonoma), you might need to:

1. **Completely quit** your terminal app (not just close the window)
2. **Remove** it from Accessibility list
3. **Restart your Mac** (yes, really)
4. **Add it back** to Accessibility list
5. **Launch terminal** again
6. **Run vox daemon**

### Using iTerm, Warp, or Another Terminal?

The app name in Accessibility should match your actual terminal:

- **Terminal.app** → "Terminal"
- **iTerm** → "iTerm2"
- **Warp** → "Warp"
- **Alacritty** → "Alacritty"

---

## Still Not Working?

Try running vox directly without the daemon first to trigger the permission prompt:

```bash
# This might trigger the permission dialog
./target/release/vox daemon
```

Then press the hotkey immediately. macOS should prompt you.

---

## Alternative: Test with a Simple Script

To verify accessibility is working, try this test:

```bash
# In a separate terminal, run:
caffeinate -d
```

Then press `Cmd+Shift+Space`. If you see logs in the vox daemon, permissions are working!

If you still see nothing, let me know:

1. What terminal app are you using?
2. Is it listed in System Settings → Privacy & Security → Accessibility?
3. Is the toggle ON?
4. Did you restart the daemon after granting permissions?
