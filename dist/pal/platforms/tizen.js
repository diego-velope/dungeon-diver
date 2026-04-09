/**
 * Samsung Tizen: key table + media key registration
 */
(function (global) {
    'use strict';

    var P = global.DungeonDiverPAL.PLATFORM_IDS;

    global.DungeonDiverPAL.registerPlatform(P.TIZEN, {
        keyMapping: {
            up: [38, 29460],
            down: [40, 29461],
            left: [37, 29462],
            right: [39, 29463],
            action: [13, 29443],
            back: [10009, 4, 27],
            media_play_pause: [415, 19],
            media_fast_forward: [417],
            media_rewind: [412]
        },
        registerKeys: function (ctx) {
            if (ctx.currentPlatform !== P.TIZEN) return;
            if (!global.tizen || !global.tizen.tvinputdevice) return;
            try {
                global.tizen.tvinputdevice.registerKeyBatch([
                    'MediaPlayPause',
                    'MediaFastForward',
                    'MediaRewind',
                    'ColorF0Red',
                    'ColorF1Green',
                    'ColorF2Yellow',
                    'ColorF3Blue'
                ]);
                console.log('[PAL] Tizen media keys registered successfully');
            } catch (e) {
                console.warn('[PAL] Tizen key registration failed:', e);
            }
        },
        shutdownHost: function () {
            if (global.tizen && global.tizen.application) {
                global.tizen.application.getCurrentApplication().exit();
                return true;
            }
            return false;
        }
    });
})(typeof window !== 'undefined' ? window : this);
