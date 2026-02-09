'use client';

import { useCallback, useMemo, useState } from 'react';

interface PermissionItem {
  id: string;
  name: string;
  code: string;
}

function formatPermissionName(permission: string): string {
  const parts = permission.split(':');
  const relevantParts = parts.slice(1);
  return relevantParts
    .map(part => part.charAt(0).toUpperCase() + part.slice(1).replace(/_/g, ' '))
    .join(' ');
}

function toPermissionItem(permission: string): PermissionItem {
  return {
    id: permission,
    name: formatPermissionName(permission),
    code: permission,
  };
}

interface UsePlanTransferStateProps {
  available: string[];
  selected: string[];
  onChange: (selected: string[]) => void;
}

export function usePlanTransferState({
  available: allAvailable,
  selected,
  onChange
}: UsePlanTransferStateProps) {
  const [leftSearch, setLeftSearch] = useState('');
  const [rightSearch, setRightSearch] = useState('');

  const availableItems = useMemo(() => {
    return allAvailable
      .filter((perm) => !selected.includes(perm))
      .map(toPermissionItem);
  }, [allAvailable, selected]);

  const selectedItems = useMemo(() => {
    return selected.map(toPermissionItem);
  }, [selected]);

  const filteredAvailable = useMemo(() => {
    const search = leftSearch.toLowerCase();
    return availableItems.filter(
      (item) =>
        item.name.toLowerCase().includes(search) ||
        item.code.toLowerCase().includes(search)
    );
  }, [availableItems, leftSearch]);

  const filteredSelected = useMemo(() => {
    const search = rightSearch.toLowerCase();
    return selectedItems.filter(
      (item) =>
        item.name.toLowerCase().includes(search) ||
        item.code.toLowerCase().includes(search)
    );
  }, [selectedItems, rightSearch]);

  const moveRight = useCallback(
    (item: PermissionItem) => {
      if (!selected.includes(item.id)) {
        onChange([...selected, item.id]);
      }
    },
    [selected, onChange]
  );

  const moveLeft = useCallback(
    (item: PermissionItem) => {
      onChange(selected.filter((id) => id !== item.id));
    },
    [selected, onChange]
  );

  return {
    leftSearch,
    setLeftSearch,
    rightSearch,
    setRightSearch,
    availableItems,
    selectedItems,
    filteredAvailable,
    filteredSelected,
    moveRight,
    moveLeft,
  };
}
