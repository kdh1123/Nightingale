export function findByIdOrFirst<T extends { id: number }>(
  items: T[] | undefined,
  id: number | null,
) {
  return items?.find((item) => item.id === id) ?? items?.[0];
}
