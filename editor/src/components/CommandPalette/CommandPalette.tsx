/**
 * AIT42 Editor - Command Palette Component
 */

import React, { useState, useEffect, useRef } from 'react';
import { Command } from '../../types';
import styles from './CommandPalette.module.css';

interface CommandPaletteProps {
  onClose: () => void;
}

/**
 * Command palette for quick actions
 *
 * Features:
 * - Fuzzy search commands
 * - Keyboard navigation
 * - Recent commands
 * - Command categories
 */
export const CommandPalette: React.FC<CommandPaletteProps> = ({ onClose }) => {
  const [search, setSearch] = useState('');
  const [selectedIndex, setSelectedIndex] = useState(0);
  const inputRef = useRef<HTMLInputElement>(null);
  const listRef = useRef<HTMLDivElement>(null);

  // Available commands
  const commands: Command[] = [
    {
      id: 'file.open',
      label: 'Open File',
      description: 'Open a file from the workspace',
      category: 'File',
      keybinding: 'Cmd+O',
      action: () => {
        console.log('Open file');
        onClose();
      },
    },
    {
      id: 'file.save',
      label: 'Save File',
      description: 'Save the current file',
      category: 'File',
      keybinding: 'Cmd+S',
      action: () => {
        console.log('Save file');
        onClose();
      },
    },
    {
      id: 'file.saveAll',
      label: 'Save All Files',
      description: 'Save all open files',
      category: 'File',
      keybinding: 'Cmd+K S',
      action: () => {
        console.log('Save all files');
        onClose();
      },
    },
    {
      id: 'view.toggleSidebar',
      label: 'Toggle Sidebar',
      description: 'Show or hide the sidebar',
      category: 'View',
      keybinding: 'Cmd+B',
      action: () => {
        console.log('Toggle sidebar');
        onClose();
      },
    },
    {
      id: 'view.toggleTerminal',
      label: 'Toggle Terminal',
      description: 'Show or hide the terminal',
      category: 'View',
      keybinding: 'Cmd+J',
      action: () => {
        console.log('Toggle terminal');
        onClose();
      },
    },
    {
      id: 'agent.run',
      label: 'Run Agent',
      description: 'Execute an AI agent',
      category: 'Agent',
      action: () => {
        console.log('Run agent');
        onClose();
      },
    },
    {
      id: 'agent.coordinator',
      label: 'Run Coordinator',
      description: 'Let the coordinator select the best agent',
      category: 'Agent',
      action: () => {
        console.log('Run coordinator');
        onClose();
      },
    },
    {
      id: 'tmux.list',
      label: 'List Tmux Sessions',
      description: 'Show all active tmux sessions',
      category: 'Tmux',
      action: () => {
        console.log('List tmux sessions');
        onClose();
      },
    },
  ];

  // Filter commands based on search
  const filteredCommands = commands.filter((cmd) => {
    const searchLower = search.toLowerCase();
    return (
      cmd.label.toLowerCase().includes(searchLower) ||
      cmd.description?.toLowerCase().includes(searchLower) ||
      cmd.category.toLowerCase().includes(searchLower)
    );
  });

  // Handle keyboard navigation
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      switch (e.key) {
        case 'ArrowDown':
          e.preventDefault();
          setSelectedIndex((prev) =>
            prev < filteredCommands.length - 1 ? prev + 1 : prev
          );
          break;
        case 'ArrowUp':
          e.preventDefault();
          setSelectedIndex((prev) => (prev > 0 ? prev - 1 : prev));
          break;
        case 'Enter':
          e.preventDefault();
          if (filteredCommands[selectedIndex]) {
            filteredCommands[selectedIndex].action();
          }
          break;
        case 'Escape':
          e.preventDefault();
          onClose();
          break;
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [filteredCommands, selectedIndex, onClose]);

  // Focus input on mount
  useEffect(() => {
    inputRef.current?.focus();
  }, []);

  // Scroll selected item into view
  useEffect(() => {
    if (listRef.current) {
      const selectedElement = listRef.current.children[selectedIndex] as HTMLElement;
      if (selectedElement) {
        selectedElement.scrollIntoView({
          block: 'nearest',
          behavior: 'smooth',
        });
      }
    }
  }, [selectedIndex]);

  // Reset selected index when search changes
  useEffect(() => {
    setSelectedIndex(0);
  }, [search]);

  return (
    <div className={styles.overlay} onClick={onClose}>
      <div className={styles.palette} onClick={(e) => e.stopPropagation()}>
        {/* Search Input */}
        <div className={styles.searchContainer}>
          <svg
            className={styles.searchIcon}
            width="16"
            height="16"
            viewBox="0 0 16 16"
            fill="currentColor"
          >
            <path d="M11.5 7a4.5 4.5 0 1 1-9 0 4.5 4.5 0 0 1 9 0zm-.82 4.74a6 6 0 1 1 1.06-1.06l3.04 3.04a.75.75 0 1 1-1.06 1.06l-3.04-3.04z" />
          </svg>
          <input
            ref={inputRef}
            type="text"
            className={styles.searchInput}
            placeholder="Type a command or search..."
            value={search}
            onChange={(e) => setSearch(e.target.value)}
          />
        </div>

        {/* Command List */}
        <div ref={listRef} className={styles.commandList}>
          {filteredCommands.length > 0 ? (
            filteredCommands.map((cmd, index) => (
              <div
                key={cmd.id}
                className={`${styles.commandItem} ${
                  index === selectedIndex ? styles.selected : ''
                }`}
                onClick={() => cmd.action()}
                onMouseEnter={() => setSelectedIndex(index)}
              >
                <div className={styles.commandInfo}>
                  <div className={styles.commandLabel}>{cmd.label}</div>
                  {cmd.description && (
                    <div className={styles.commandDescription}>
                      {cmd.description}
                    </div>
                  )}
                </div>
                <div className={styles.commandMeta}>
                  <span className={styles.commandCategory}>{cmd.category}</span>
                  {cmd.keybinding && (
                    <kbd className={styles.commandKeybinding}>{cmd.keybinding}</kbd>
                  )}
                </div>
              </div>
            ))
          ) : (
            <div className={styles.noResults}>
              <p>No commands found</p>
            </div>
          )}
        </div>
      </div>
    </div>
  );
};
