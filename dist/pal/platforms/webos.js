/**
 * LG webOS
 */
(function (global) {
    'use strict';

    var P = global.DungeonDiverPAL.PLATFORM_IDS;

    global.DungeonDiverPAL.registerPlatform(P.WEBOS, {
        keyMapping: {
            up: [38],
            down: [40],
            left: [37],
            right: [39],
            action: [13],
            back: [461]
        },
        shutdownHost: function () {
            if (global.webOS && typeof global.webOS.platformBack === 'function') {
                global.webOS.platformBack();
                return true;
            }
            return false;
        }
    });
})(typeof window !== 'undefined' ? window : this);
