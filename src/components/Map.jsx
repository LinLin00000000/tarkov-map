import { useEffect, useRef, useMemo, useCallback, useState } from 'react';
import { useParams } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import {
    TransformWrapper,
    TransformComponent,
} from 'react-zoom-pan-pinch';
import { MapContainer, LayersControl } from 'react-leaflet'
import L from 'leaflet';

import { useMapImages } from '../features/maps';

import Time from './Time';
import SEO from './SEO';

import ErrorPage from './error-page';

const showTestMarkers = true;

const testMarkers = {
    customs: [
        {
            name: '0, 0',
            coordinates: [0, 0]
        },
        {
            name: 'Flash drive with fake info',
            coordinates: [-91.521, 4.346]
        },
        {
            name: 'Bronze pocket watch',
            coordinates: [102.2964, -5.9064]
        },
        {
            name: 'Secure Folder 0022',
            coordinates: [-202.8806, -102.4012]
        },
        {
            name: 'Secure Folder 0031',
            coordinates: [204.073, 12.3975]
        },
        {
            name: 'Secure Folder 0048',
            coordinates: [367.6828, -50.5712]
        },
        {
            name: 'Golden Zibbo',
            coordinates: [180.8072, 150.0781]
        },
        {
            name: 'Secure Folder 0013',
            coordinates: [498.5496, -141.5012]
        },
        {
            name: 'Carbon case',
            coordinates: [239.183, 160.595]
        },
        {
            name: 'Sliderkey Secure Flash Drive',
            coordinates: [194.2243, 171.0656]
        },
        {
            name: 'Package of graphics cards',
            coordinates: [-204.388, -98.63]
        }
    ],
    lighthouse: [
        {
            name: 'Water pump operation data',
            coordinates: [-111.0176, -750.147034]
        },
        {
            name: 'Pumping station operation report',
            coordinates: [53.2094955, -638.8754]
        },
        {
            name: 'Laptop with information',
            coordinates: [139.196289, -129.943909]
        },
        {
            name: 'Stolen military documents',
            coordinates: [-115.694664, 88.8879852]
        },
        {
            name: 'Sealed letter',
            coordinates: [-248.391052, -328.2831]
        },
        {
            name: 'Forged Lightkeeper intelligence',
            coordinates: [21.2779083, -445.487946]
        },
        {
            name: 'Working hard drive',
            coordinates: [352.6155, 545.5325]
        },
    ],
    reserve: [/*
        {
            name: 'Military documents #1',
            coordinates: [-114.36499, 27.3590088]
        },
        {
            name: 'Military documents #2',
            coordinates: [-114.18103, 39.7880249]
        },
        {
            name: 'Military documents #3',
            coordinates: [-121.867004, 38.6560059]
        },
        {
            name: 'MBT Integrated Navigation System',
            coordinates: [99.78601, 59.1600342]
        },
        {
            name: 'T-90M Commander Control Panel',
            coordinates: [-88.065, 151.055]
        },
        {
            name: 'Medical record #1',
            coordinates: [-59.0209961, -36.1919861]
        },
        {
            name: 'Medical record #2',
            coordinates: [-80.47601, -30.7659912]
        },
        {
            name: 'Lightkeeper intelligence',
            coordinates: [102.24981, 65.5604]
        },
    */],
    woods: [
        {
            name: 'Encrypted message',
            coordinates: [-257.1476, 7.9998]
        },
        {
            name: 'Blood sample',
            coordinates: [-94.25195, 218.992981]
        },
        {
            name: 'Secure folder 0052',
            coordinates: [-3.322, -81.1259]
        },
        {
            name: 'Motor Controller #1',
            coordinates: [233.843018, -71.10199]
        },
        {
            name: 'Single-axis Fiber Optic Gyroscope #1',
            coordinates: [56.7209473, -50.2489929]
        },
    ],
};

function Map() {
    let { currentMap } = useParams();

    const { t } = useTranslation();

    useEffect(() => {
        let viewableHeight = window.innerHeight - document.querySelector('.navigation')?.offsetHeight || 0;
        if (viewableHeight < 100) {
            viewableHeight = window.innerHeight;
        }

        document.documentElement.style.setProperty(
            '--display-height',
            `${viewableHeight}px`,
        );

        return function cleanup() {
            document.documentElement.style.setProperty(
                '--display-height',
                `auto`,
            );
        };
    });

    const ref = useRef();

    const [mapRef, setMapRef] = useState(null);
    const onMapRefChange = useCallback(node => {
        setMapRef(node);
    }, []);

    const [legend, setLegendRef] = useState(null);
    const onLegendRefChange = useCallback(node => {
        setLegendRef(node);
    }, []);

    useEffect(() => {
        ref?.current?.resetTransform();
    }, [currentMap]);

    let allMaps = useMapImages();

    const mapData = useMemo(() => {
        return allMaps[currentMap];
    }, [allMaps, currentMap]);

    const transformation = useMemo(() => {
        if (!mapData || !mapData.transform) {
            return new L.Transformation(1, 0, 1, 0);
        }
        let scaleX = mapData.transform[0];
        let scaleY = mapData.transform[2];
        let marginX = mapData.transform[1];
        let marginY = mapData.transform[3];
        if (mapData.coordinateRotation === 90) {
            //factory
        }
        if (mapData.coordinateRotation === 180) {
            scaleX = scaleX * -1;
            scaleY = scaleY * -1;
        }
        if (mapData.coordinateRotation === 270) {
            //labs
        }
        return new L.Transformation(scaleX, marginX, scaleY, marginY);
    }, [mapData]);

    useEffect(() => {
        if (!mapRef || !legend || !mapData || !mapData.tileSize) {
            return;
        }
        while (legend._layers.length > 0) {
            legend.removeLayer(legend._layers[0].layer)
        }
        mapRef.eachLayer(layer => layer?.remove());
        mapRef.setMinZoom(mapData.minZoom);
        mapRef.setMaxZoom(mapData.maxZoom);
        const baseLayer = L.tileLayer(mapData.mapPath || `https://assets.tarkov.dev/maps/${mapData.normalizedName}/{z}/{x}/{y}.png`, {tileSize: mapData.tileSize});
        baseLayer.addTo(mapRef);
        const markers = testMarkers[mapData.normalizedName];
        if (showTestMarkers && markers) {
            const markerLayer = L.layerGroup();
            for (const m of markers) {
                const point = transformation.transform(L.point(m.coordinates[0], m.coordinates[1]));
                L.marker([point.y, point.x])
                    .bindPopup(L.popup().setContent(m.name))
                    .addTo(markerLayer);
            }
            if (markers.length > 0) {
                markerLayer.addTo(mapRef);
                legend.addOverlay(markerLayer, t('Markers'));
            }
        }
        if (mapData.layers) {
            for (const layer of mapData.layers) {
                const tileLayer = L.tileLayer(layer.path, {tileSize: mapData.tileSize});
                legend.addOverlay(tileLayer, t(layer.name));
                if (layer.show) {
                    tileLayer.addTo(mapRef);
                }
            }
        } 
        const zeroPoint = transformation.transform(L.point(0, 0));
        mapRef.panTo([zeroPoint.y, zeroPoint.x])
    }, [mapData, mapRef, transformation, legend, t]);
    
    if (!mapData) {
        return <ErrorPage />;
    }

    return [
        <SEO 
            title={`${mapData.displayText} - ${t('Escape from Tarkov')} - ${t('Tarkov.dev')}`}
            description={mapData.description}
            image={`${window.location.origin}${process.env.PUBLIC_URL}${mapData.imageThumb}`}
            card='summary_large_image'
            key="seo-wrapper"
        />,
        <div className="display-wrapper" key="map-wrapper" style={{height: '500px'}}>
            <Time
                currentMap={currentMap}
                normalizedName={mapData.normalizedName}
                duration={mapData.duration}
                players={mapData.players}
                author={mapData.author}
                authorLink={mapData.authorLink}
            />
            {mapData.projection !== 'interactive' && (<TransformWrapper
                ref={ref}
                initialScale={1}
                centerOnInit={true}
                wheel={{
                    step: 0.1,
                }}
            >
                <TransformComponent>
                    <div className="map-image-wrapper">
                        <img
                            alt={`${mapData.displayText} ${t('Map')}`}
                            loading="lazy"
                            className="map-image"
                            title={`${mapData.displayText} ${t('Map')}`}
                            src={`${process.env.PUBLIC_URL}${mapData.image}`}
                        />
                    </div>
                </TransformComponent>
            </TransformWrapper>)}
            {mapData.projection === 'interactive' && (<MapContainer ref={onMapRefChange} center={[0, 0]} zoom={2} scrollWheelZoom={true} crs={L.CRS.Simple} style={{height: '500px', backgroundColor: 'transparent'}}>
                <LayersControl
                    ref={onLegendRefChange}
                    position="bottomleft"
                ></LayersControl>
            </MapContainer>)}
        </div>,
    ];
}
export default Map;
