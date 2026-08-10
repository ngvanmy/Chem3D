import React from 'react';
import { Canvas } from '@react-three/fiber';
import { OrbitControls } from '@react-three/drei';
import * as THREE from 'three';

interface Atom {
  element: string;
  x: number;
  y: number;
  z: number;
}

interface Bond {
  source: number;
  target: number;
  type: number;
}

interface MoleculeViewerProps {
  atoms: Atom[];
  bonds: Bond[];
}

const ELEMENT_COLORS: Record<string, string> = {
  H: '#ffffff', // White
  C: '#808080', // Gray
  O: '#ff0000', // Red
  N: '#0000ff', // Blue
  S: '#ffff00', // Yellow
  P: '#ff8000', // Orange
  F: '#00ff00',
  Cl: '#00cc00',
  Br: '#990000',
  I: '#660099',
};

const ELEMENT_RADII: Record<string, number> = {
  H: 0.3,
  C: 0.6,
  O: 0.5,
  N: 0.55,
  S: 0.7,
  P: 0.65,
};

const CylinderBond: React.FC<{ start: [number, number, number]; end: [number, number, number] }> = ({ start, end }) => {
  const pStart = new THREE.Vector3(...start);
  const pEnd = new THREE.Vector3(...end);
  const distance = pStart.distanceTo(pEnd);
  const position = pStart.clone().add(pEnd).multiplyScalar(0.5);

  const direction = new THREE.Vector3().subVectors(pEnd, pStart).normalize();
  const up = new THREE.Vector3(0, 1, 0);
  const quaternion = new THREE.Quaternion().setFromUnitVectors(up, direction);

  return (
    <mesh position={position} quaternion={quaternion}>
      <cylinderGeometry args={[0.12, 0.12, distance, 8]} />
      <meshStandardMaterial color="#b0b0b0" metalness={0.2} roughness={0.3} />
    </mesh>
  );
};

export const MoleculeViewer: React.FC<MoleculeViewerProps> = ({ atoms, bonds }) => {
  return (
    <div className="w-full h-[400px] bg-slate-950 rounded-xl overflow-hidden border border-slate-800 relative">
      <Canvas camera={{ position: [0, 0, 10], fov: 60 }}>
        <ambientLight intensity={0.5} />
        <directionalLight position={[10, 10, 10]} intensity={1} castShadow />
        <directionalLight position={[-10, -10, -10]} intensity={0.3} />
        <group>
          {atoms.map((atom, idx) => {
            const color = ELEMENT_COLORS[atom.element] || '#ff00ff';
            const radius = ELEMENT_RADII[atom.element] || 0.5;
            return (
              <mesh key={idx} position={[atom.x, atom.y, atom.z]}>
                <sphereGeometry args={[radius, 32, 32]} />
                <meshStandardMaterial color={color} roughness={0.2} metalness={0.1} />
              </mesh>
            );
          })}
          {bonds.map((bond, idx) => {
            const startAtom = atoms[bond.source];
            const endAtom = atoms[bond.target];
            if (!startAtom || !endAtom) return null;
            return (
              <CylinderBond
                key={idx}
                start={[startAtom.x, startAtom.y, startAtom.z]}
                end={[endAtom.x, endAtom.y, endAtom.z]}
              />
            );
          })}
        </group>
        <OrbitControls enableDamping dampingFactor={0.05} />
      </Canvas>
      <div className="absolute bottom-3 left-3 bg-slate-900/80 px-3 py-1 rounded text-xs text-slate-400 border border-slate-800">
        Dùng chuột để Xoay / Cuộn để Zoom
      </div>
    </div>
  );
};
