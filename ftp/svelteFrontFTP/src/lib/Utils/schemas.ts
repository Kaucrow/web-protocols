
import {  z } from 'zod';

// Definir el esquema IFile
export const IFileSchema = z.object({
  name: z.string(),
  type: z.enum(['file', 'directory']),
  size: z.string().optional(),
  date: z.string().optional()
});


