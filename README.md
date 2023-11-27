# Webcam streamer

Intended for streaming from a small ARM linux device like a raspberry pi on the local network, turning a USB camera (video4linux2) into an IP camera.
I.e. naive version of [mjpg-streamer](https://github.com/jacksonliam/mjpg-streamer).

Start the streamer with (optional) args `<video dev id> <port> <pass>`, e.g. `cargo r -r 0 3333 TEST` (in its folder) on the source device, then run the client elsewhere on the local network (not otherwise secure).
