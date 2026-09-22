import QtQuick 2.5
import calamares.slideshow 1.0

Presentation {
    id: presentation

    function onActivate() { presentation.currentSlide = 0 }
    function onLeave() { }

    Slide {
        Text {
            anchors.centerIn: parent
            text: "Balsa is installing.\n\nYour configuration is being written to disk\nas files you can read and rebuild from.\n\nPlease note that the installer may freeze during \"balsa-install\".\nThis is normal; it builds from source."
            horizontalAlignment: Text.AlignHCenter
            font.pixelSize: 22
            wrapMode: Text.WordWrap
        }
    }

    Slide {
        Text {
            anchors.centerIn: parent
            text: "Tuning profiles are switchable.\n\nPick a different one at boot,\nno reinstall needed."
            horizontalAlignment: Text.AlignHCenter
            font.pixelSize: 22
            wrapMode: Text.WordWrap
        }
    }

    Slide {
        Text {
            anchors.centerIn: parent
            text: "balsa: Lightweight, malleable Linux, powered by Nix."
            horizontalAlignment: Text.AlignHCenter
            font.pixelSize: 22
            wrapMode: Text.WordWrap
        }
    }
}
