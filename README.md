# Webcam streamer

Intended for streaming from a small ARM linux device like a raspberry pi on the local network, turning a USB camera (video4linux2) into an IP camera.
I.e. naive version of [mjpg-streamer](https://github.com/jacksonliam/mjpg-streamer).

Start streamer on the source device, then run the client elsewhere on the local network (not otherwise secure).
Start the streamer with `cargo r -r --bin webcam_viewer_streamer`, append `-- --help` for options.
Defaults to local, pass `--ip 0.0.0.0` for network visibility.
