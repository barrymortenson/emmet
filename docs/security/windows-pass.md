# The Windows pass (H-10)

Barry's hands-on pass on the Windows 11 VM, about an hour. It checks what CI's hosted Windows machines can't: a
real person's account, a real graphics card, the real agents, and the installer's own windows. It also covers the
Windows halves that sessions 1F, 4A, 4B, 4C, and 4E left for this pass.

Each step says what you should see. Tick it, or write what you saw instead. Where a step asks for a paste or a
screenshot, the reason is beside it. Stop at the first step that fails and tell the agent; it fixes it before the
PR opens.

## Before you start (on the Mac)

1. Download the installer the PR's CI built. The agent gives you the one-line `gh run download` command with the
   run's number; it lands in `~/Downloads/emmet-windows-nsis/`.
   - [ ] You see `Emmet_0.1.0_x64-setup.exe` there.

## A. A fresh account with a name Windows users really have (5 min)

Why: a user folder with a space and an accent is the commonest source of Windows-only bugs, and a new account is the
state a new user starts from (1F's fresh-account run).

2. In the VM: Settings → Accounts → Other users → Add account → "I don't have this person's sign-in information" →
   "Add a user without a Microsoft account". Name it `Zoë Test`, no password.
3. Sign out, and sign in as Zoë Test.
   - [ ] File Explorer shows the user folder `C:\Users\Zoë Test`.

## B. Install (5 min)

4. Open `\\Mac\Home\Downloads\emmet-windows-nsis` in File Explorer and double-click the installer.
   - [ ] SmartScreen says "Windows protected your PC" (the installer is unsigned until 6A). "More info" → "Run
     anyway" starts it.
   - [ ] No "Do you want to allow this app to make changes" prompt appears: it installs for this user only.
5. Click through the installer with its defaults and leave "Run Emmet" ticked.
   - [ ] Emmet opens on its first-run screen.
   - [ ] The Start menu has Emmet.

## C. First run (5 min)

6. Start a stopwatch and click Get started.
   - [ ] A progress bar shows the parts library downloading, then Python setting up.
   - [ ] The Cottage appears, built. Write the time: ______ (the budget is 3 minutes).
7. Drag in the viewer to orbit for a few seconds, then scroll to zoom.
   - [ ] The model turns smoothly, with no white flashes or missing parts.

## D. The window, the menus, and the keys (5 min)

Why: Windows draws its own title row and menus (macOS uses the system's), and they have only been seen on CI.

8. Look at the title row.
   - [ ] The project chip, then File, Edit, View, Agent, Help, the view switch (Iso, Front, Top), and minimize,
     maximize, and close at the right. Nothing overlaps.
9. Drag the window's right edge in until the window stops getting narrower.
   - [ ] File to Help never slide under the view switch.
10. Click File.
    - [ ] It lists New, Open…, Open recent with the Cottage under it, Save version, Versions…, Export…, Settings…,
      and Exit with Alt+F4 beside it.
11. Press Esc, then try each: Ctrl+1 to Ctrl+4, Ctrl+, (Settings; Esc closes it), F11 twice, and 1 to 6 with the
    pointer over the viewer.
    - [ ] The panel's tabs change; Settings opens and closes; the window fills the screen and comes back; the view
      changes.
12. Double-click an empty part of the title row, twice.
    - [ ] The window maximizes, then restores.
    Then press Tab a dozen times.
    - [ ] Each control it lands on shows a focus ring (CI can't check this on Windows).

## E. Screen scale (5 min)

Why: Windows laptops run at 125 to 200 %, and the viewer has to follow a change without a restart.

13. With Emmet open, set Settings → System → Display → Scale to 100 %, then 150 %, then 200 % (Parallels may ask to
    keep the change).
    - [ ] At each, Emmet's text is sharp, nothing is cut off or overlapping, and the model's edges are sharp, not
      blurry or blocky.
14. Set the scale back to what it was.

## F. Sliders, checks, and speed (5 min)

15. In the Options tab, drag a Tweaks slider (Weathering) a few steps.
    - [ ] The model changes within about a second each time, and the camera doesn't move.
16. Click Replay build (the play button in the title row) and let it finish.
    - [ ] The Cottage builds up part by part, smoothly.
17. Orbit for five seconds, then Help → Copy diagnostics, and paste into a Notepad file saved as
    `\\Mac\Home\Downloads\emmet-diagnostics.txt`.
    Why: the text has the graphics card's name, the quality Emmet picked, and its frame and load times: the Windows
    numbers the PR needs for section 13's budgets, from a machine with a real GPU.
    - [ ] The file has a `Graphics:` line that doesn't say SwiftShader or Basic Render, and a `Last motion:` line.
18. Click the Checks tab.
    - [ ] It says no problems (or lists findings; clicking one highlights its parts in the viewer).

## G. Both agents (15 min)

19. In the chat, pick Claude and follow its Set up (install and sign in, with no terminal).
    - [ ] Claude Code installs and signs in with your plan, and the badge names the plan.
20. Send: `Make the roof blue.`
    - [ ] Claude works, the roof turns blue, and the reply is in plain words. If a card asks to allow a command,
      allow it once.
21. Start a new model: File → New, name it `Hut`, and send: `Build a small hut with a door, two windows, and a
    pitched roof.` Give it up to 10 minutes.
    - [ ] A hut appears with a door, windows, and a roof; Checks says no problems, or the agent fixes what it finds.
22. Switch the agent to Codex and follow its Set up, then send: `Make the door red.`
    - [ ] Codex works and the door turns red.

## H. Exports (5 min)

23. Ctrl+E → CSV, then Ctrl+E → LDraw .mpd, then Ctrl+E → PDF.
    - [ ] Each says Saved … with a "Show in folder" button, and the three files are in Zoë's Downloads.
    - [ ] The PDF opens and shows the hut's steps with its parts.

## I. A project from somewhere else (3 min)

24. In File Explorer, copy the `Hut` folder from Documents\Emmet to the Desktop. In Emmet: File → Open…, and pick
    the copy's `emmet.toml`.
    - [ ] It opens read-only with a banner and a Trust this project button; the chat refuses a message until you
      trust it.
25. Click Trust this project.
    - [ ] It builds, and the banner goes.

## J. Install over itself, and uninstall (7 min)

Why: the real updater comes in 6A; until then an update is the installer run again. Either way your data must stay.

26. Close Emmet and run the installer again, with its defaults.
    - [ ] Emmet opens straight to your last project: no first run, the agents still set up.
27. Close Emmet. Settings → Apps → Installed apps → Emmet → Uninstall, and leave "Delete the application data"
    unticked.
    - [ ] Emmet is gone from the Start menu.
    - [ ] `%APPDATA%\com.emmetbuilder.emmet` still exists (paste that into File Explorer's address bar), and so
      does Documents\Emmet with your projects.
28. Install once more, close Emmet, and uninstall again with "Delete the application data" ticked.
    - [ ] `%APPDATA%\com.emmetbuilder.emmet` is gone. Documents\Emmet, your projects, is still there.

## K. Clean up

29. Sign out of Zoë Test and sign in as yourself; delete the Zoë Test account if you like (Settings → Accounts →
    Other users → Remove).

## Two things to look at while you're there

Both were seen in CI screenshots at a small window; say whether they show at a normal size.

- [ ] The first-run welcome card (step 5) has no scrollbar at the window's normal size.
- [ ] With a version open to compare (the project chip → Versions… → pick one, after Save version), the bar with
  Restore, Open as new project, the version's name, and close isn't cut off under the panel.

## What to send back

- The ticks, or what you saw where a step didn't go as written.
- The first-run time from step 6.
- `emmet-diagnostics.txt` from step 17 (it has the folder paths of the Zoë Test account only).

## Results

2026-11-14, Barry, on the Windows 11 ARM VM (Parallels), with the installer from CI run 43490461801: every line
ticked, 46 of 46, with no notes. That covers each step above, the Tab focus rings, and the two things to look at
(no scrollbar on the welcome card, and the version bar not cut off, at the window's normal size).
