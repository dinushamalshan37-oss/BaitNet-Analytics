"use client";

import { MapContainer, Marker, Popup, TileLayer } from "react-leaflet";
import type { LatLngExpression } from "leaflet";
import "leaflet/dist/leaflet.css";

export function GeoMap({ latitude, longitude, label }: { latitude: number; longitude: number; label: string }) {
  const position: LatLngExpression = [latitude, longitude];
  return (
    <div className="overflow-hidden rounded-xl border border-line">
      <MapContainer center={position} zoom={5} scrollWheelZoom={false} className="h-72 w-full">
        <TileLayer attribution='&copy; <a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a>' url="https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png" />
        <Marker position={position}><Popup>{label}</Popup></Marker>
      </MapContainer>
    </div>
  );
}
