/** Shared pet nesting for the main meter and overlay.
 *  Companion's petRows rule: one helper, every surface. The engine still
 *  folds pet damage onto the owner as `Pet (Name): ability` — this module
 *  is the only place those strings get regrouped for display. */

export function petNameFromAbility(abilityName: string): string | null {
  const match = /^Pet \((.+)\): /.exec(abilityName);
  if (!match) return null;
  return match[1];
}

export function isPetAbility(abilityName: string): boolean {
  return petNameFromAbility(abilityName) != null;
}

/** Sum damage per pet from namespaced ability rows. */
export function petDamageByName(
  abilities: { name: string; damage: number }[]
): Map<string, number> {
  const pets = new Map<string, number>();
  for (const ability of abilities) {
    const petName = petNameFromAbility(ability.name);
    if (!petName) continue;
    const existing = pets.get(petName) ?? 0;
    pets.set(petName, existing + ability.damage);
  }
  return pets;
}

type AbilityLike = {
  name: string;
  hits: number;
  damage: number;
  healing: number;
};

/** Collapse `Pet (Name): slash` + `Pet (Name): cleave` into one nested row per pet. */
export function nestPetAbilities<T extends AbilityLike>(abilities: T[]): T[] {
  const pets = new Map<string, T>();
  const own: T[] = [];

  for (const ability of abilities) {
    const petName = petNameFromAbility(ability.name);
    if (!petName) {
      own.push(ability);
      continue;
    }
    const key = petName.toLowerCase();
    const existing = pets.get(key);
    if (existing) {
      existing.hits += ability.hits;
      existing.damage += ability.damage;
      existing.healing += ability.healing;
    } else {
      pets.set(key, {
        ...ability,
        name: `Pet (${petName})`,
      });
    }
  }

  const combined = [...own, ...pets.values()];
  return combined;
}
