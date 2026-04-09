/**
 * Amazon Fire TV / Fire TV-flavored Android
 */
(function (global) {
    'use strict';

    var P = global.DungeonDiverPAL.PLATFORM_IDS;

    global.DungeonDiverPAL.registerPlatform(P.FIRETV, {
        keyMapping: {
            up: [19],
            down: [20],
            left: [21],
            right: [22],
            action: [23, 66],
            back: [4, 27, 8, 11]
        },
        shutdownHost: function () {
            if (typeof global.AndroidJsInterface !== 'undefined' &&
                typeof global.AndroidJsInterface.shutdown === 'function') {
                global.AndroidJsInterface.shutdown();
                return true;
            }
            return false;
        }
    });
})(typeof window !== 'undefined' ? window : this);
