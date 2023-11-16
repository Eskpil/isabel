# Isabel

Flutter is an awesome framework with an even better ecosystem. Currently, it's builtin linux support is lacking since it
builds on [GTK](https://gtk.org). GTK is great if you are just trying to ship something on linux but when your 
priorities are linux and more specifically wayland. You start to become frustrated.

Isabel benefits from the flutter embedding capabilities and mounts flutter applications to the wayland compositor using 
[Aylin](https://github.com/Eskpil/aylin). Both Isabel and Aylin are in their beginning faces. And Isabel is the primary 
driver of Aylin development.

## Goal

The goal is to be able to create native wayland experiences with flutter. For example, with Isabel flutter can be used
with the [wlr-layer-shell](https://wayland.app/protocols/wlr-layer-shell-unstable-v1) protocol. Layer shell is a protocol
tailor-made for making desktop widgets and utils. 