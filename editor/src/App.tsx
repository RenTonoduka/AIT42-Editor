/**
 * AIT42 Editor - Main Application Component
 */

import React from 'react';
import { Layout } from './components/Layout/Layout';
import './styles/global.css';

/**
 * Main application component
 *
 * Provides the root layout and global styles
 */
const App: React.FC = () => {
  return (
    <div className="cursor-dark-theme">
      <Layout />
    </div>
  );
};

export default App;
