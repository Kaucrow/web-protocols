export interface IFile {
  name: string;
  type: 'file' | 'directory';
  size?: string;
  date?: string;
}