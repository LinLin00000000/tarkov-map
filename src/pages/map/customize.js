import L from 'leaflet';
import React, { useEffect, useCallback, Suspense } from 'react';
import { Routes, Route, useNavigate, Navigate } from 'react-router-dom';

import Loading from '../../components/loading/index.js';
import MenuItem from '../../components/menu/MenuItem.jsx';
import { Link } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import { mapIcons, useMapImagesSortedArray } from '../../features/maps/index.js';
import rawMapData from '../../data/maps.json';
const Maps = React.lazy(() => import('../maps//index.js'));
L.Control.SelectMap = L.Control.extend({
    onAdd: function (map) {
        const div = L.DomUtil.create('div');
        
        div.classList = 'submenu-wrapper submenu-items overflow-member'
        div.innerHTML = document.querySelector('#root > div > nav > ul > li.submenu-wrapper.submenu-items.overflow-member').innerHTML

        // div.innerHTML = [
        //     '<ul>',
        //     ...rawMapData.map(
        //         (e) => `<li><a href="/map/${e.normalizedName}">${e.normalizedName}</a></li>`,
        //     ),
        //     '</ul>'
        // ].join('');

        return div;
    },

    onRemove: function (map) {
        // Nothing to do here
    },
});

L.control.selectMap = function (opts) {
    return new L.Control.SelectMap(opts);
};
