#!/bin/sh

ARCH=$( uname -m )

OS=$( uname -s )

PLATFORM=${OS}-${ARCH}

case "$PLATFORM" in
	Linux-x86_64)
		BINARY=download-linux-x64
		;;
	
	Linux-aarch64)
		BINARY=download-linux-arm64
		;;
	
	Darwin-arm64)
		BINARY=download-osx-arm64
		;;
	
	*)
		echo "Unknown platform."
		exit 1
		;;
esac

if which curl ; then
	curl -sSfL "https://github.com/VOICEVOX/voicevox_core/releases/latest/download/${BINARY}" -o ./download
elif which wget ; then
	wget "https://github.com/VOICEVOX/voicevox_core/releases/latest/download/${BINARY}" -O ./download
else
	echo "cURL or wget is required."
	exit 1
fi

chmod +x ./download

exec ./download

