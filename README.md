straight up "selfing it". and by "it", haha, well. let's justr say. My box

# Straight up running it

## Dependencies
- [scrcpy](https://scrcpy.dev/)
- [v4l2loopback](https://github.com/v4l2loopback/v4l2loopback)

## First build
### `.env` config
- Create a config file in `src-tauri/`, example:
```
VIDEO_DEVICE=/dev/video0
FPS=24
```
### Installing npm dependencies
- `cd src/ ; npm i`

## Regular build
- Use `./build.sh rebuild` for a release build, or `./build.sh` for just running it (you need to rebuild after changing `.env`)
- Use `cd src/ ; npm run tauri dev` for a dev build