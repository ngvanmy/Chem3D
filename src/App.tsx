import { MoleculeViewer } from './components/MoleculeViewer';

const mockAtoms = [
  { element: 'C', x: -0.74, y: 0.36, z: 0.00 },
  { element: 'C', x: 0.74, y: -0.36, z: 0.00 },
  { element: 'O', x: 1.83, y: 0.56, z: 0.00 },
  { element: 'H', x: -1.27, y: 0.01, z: 0.88 },
  { element: 'H', x: -0.74, y: 1.45, z: 0.00 },
  { element: 'H', x: -1.27, y: 0.01, z: -0.88 },
  { element: 'H', x: 0.74, y: -1.00, z: -0.88 },
  { element: 'H', x: 0.74, y: -1.00, z: 0.88 },
  { element: 'H', x: 2.62, y: 0.01, z: 0.00 },
];

const mockBonds = [
  { source: 0, target: 1, type: 1 },
  { source: 1, target: 2, type: 1 },
  { source: 0, target: 3, type: 1 },
  { source: 0, target: 4, type: 1 },
  { source: 0, target: 5, type: 1 },
  { source: 1, target: 6, type: 1 },
  { source: 1, target: 7, type: 1 },
  { source: 2, target: 8, type: 1 },
];

function App() {
  return (
    <div className="min-h-screen bg-slate-900 text-white flex flex-col items-center justify-center p-6">
      <h1 className="text-3xl font-bold mb-6">Chem3D-Tauri 3D Molecule Preview</h1>
      <div className="w-full max-w-2xl">
        <MoleculeViewer atoms={mockAtoms} bonds={mockBonds} />
      </div>
    </div>
  );
}

export default App;
