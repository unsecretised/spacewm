# SpaceWM - MacOS WM

Space WM is a WM where each workspace stores 2 windows. Each workspace is given the ability to store 2 windows,
a focussed and an unfocussed window. This WM is good for smaller screen macs since it allows for
 each window to take up as much screen real estate as possible. 

> [!IMPORTANT]  
> SpaceWM is still in development and doesn't have any binaries ready for distribution yet, and shouldn't be used by anyone at this stage of development

## Project goal:
To make a WM for MacOS that provides a easy to use and configure WM for macos that works across displays and doesn't require disabling SIP.

## How SpaceWM works:
- On start, each window gets a single workspace (or rule based assignment - not yet added)
- Moving a window to a workspace will either assign it to the first empty slot or swap it out to the current workspace (so the minimum number of workspaces at any given number of time is n/2 where n is the number of windows open)

## Contributing:
- Issues: [See open issues](https://github.com/unsecretised/spacewm/issues)
- Implement planned features:
  - Menu bar icon
  - Query info in json format
  - 3 finger scroll


