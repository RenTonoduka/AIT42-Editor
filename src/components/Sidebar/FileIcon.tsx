/**
 * FileIcon Component - Display file type icons
 *
 * Maps file extensions to appropriate icons
 */
import {
  FileText,
  FileCode,
  FolderClosed,
  FolderOpen,
  FileJson,
  FileImage,
  Settings,
  Package,
  type LucideIcon,
} from 'lucide-react';

interface FileIconProps {
  name: string;
  isDirectory: boolean;
  isExpanded?: boolean;
  className?: string;
}

/**
 * Get icon for file based on extension
 */
function getFileIcon(filename: string): LucideIcon {
  const ext = filename.split('.').pop()?.toLowerCase() || '';

  const iconMap: Record<string, LucideIcon> = {
    // Code files
    ts: FileCode,
    tsx: FileCode,
    js: FileCode,
    jsx: FileCode,
    rs: FileCode,
    py: FileCode,
    go: FileCode,
    c: FileCode,
    cpp: FileCode,
    h: FileCode,
    hpp: FileCode,
    java: FileCode,
    cs: FileCode,
    php: FileCode,
    rb: FileCode,
    swift: FileCode,
    kt: FileCode,

    // Config files
    json: FileJson,
    toml: Settings,
    yaml: Settings,
    yml: Settings,
    xml: Settings,
    ini: Settings,
    conf: Settings,

    // Package files
    lock: Package,

    // Images
    png: FileImage,
    jpg: FileImage,
    jpeg: FileImage,
    gif: FileImage,
    svg: FileImage,
    ico: FileImage,
    webp: FileImage,
  };

  return iconMap[ext] || FileText;
}

/**
 * Get color for file icon based on type
 */
function getIconColor(filename: string): string {
  const ext = filename.split('.').pop()?.toLowerCase() || '';

  const colorMap: Record<string, string> = {
    // TypeScript - Blue
    ts: 'text-blue-400',
    tsx: 'text-blue-400',

    // JavaScript - Yellow
    js: 'text-yellow-400',
    jsx: 'text-yellow-400',

    // Rust - Orange
    rs: 'text-orange-500',

    // Python - Blue
    py: 'text-blue-300',

    // Go - Cyan
    go: 'text-cyan-400',

    // JSON - Green
    json: 'text-green-400',

    // Config - Gray
    toml: 'text-gray-400',
    yaml: 'text-gray-400',
    yml: 'text-gray-400',

    // Images - Purple
    png: 'text-purple-400',
    jpg: 'text-purple-400',
    jpeg: 'text-purple-400',
    svg: 'text-purple-400',
  };

  return colorMap[ext] || 'text-gray-300';
}

export function FileIcon({
  name,
  isDirectory,
  isExpanded = false,
  className = '',
}: FileIconProps) {
  if (isDirectory) {
    const Icon = isExpanded ? FolderOpen : FolderClosed;
    return <Icon className={`${className} text-yellow-500`} size={16} />;
  }

  const Icon = getFileIcon(name);
  const color = getIconColor(name);

  return <Icon className={`${className} ${color}`} size={16} />;
}
