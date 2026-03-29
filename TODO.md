Process that works:
- Remove all code that handles UI events
- Remove all stuff from lichee (reset script)
- plug out the headset adapter
- reboot
- compile and copy to lichee
- open player with `audioplayer` script
- Settings, Play-Test (may take some attempts)
- After this, the listing of items should work


Slint issues
- @children cannot be in an if
- There is no `else` in Slint component renderings

todo:
- player.rs - replace tokio::select with 2 threads (1 for receiving commands and 1 for updating the time in the player)
- [ ] (Modal) Overlay
    - Player stays unconnected until player action is triggered
    - If player can't connect to sink on action, a modal message is shown
- [ ] restructure directory structure
    - nanowave/
        - nanowave.db - Database
        - nanowave.json - Preferences
        -
- [ ] touch event resets display auto shutdown
- [] Improve back and forward (e.g. by double tap?)
- [] Change logo / bootscreen
- [x] Navigation:
    - Swipe top to bottom: Navigate to settings
    - Swipe bottom to top: Navigate to player