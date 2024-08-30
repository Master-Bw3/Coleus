```json
{
  "title": "Inventory Information",
  "icon": "minecraft:bundle",
  "category": "trickster:delusions_ingresses"
}
```

Tricks that pull information from the caster's inventory.

;;;;;

<|glyph@trickster:templates|trick-id=trickster:get_item_in_slot,title=Fence's Ingress|>

slot -> item

---

Returns the type of item that the given slot contains.

;;;;;

<|glyph@trickster:templates|trick-id=trickster:other_hand,title=Juggling Delusion|>

-> item

---

Returns the type of item in the caster's other hand.

;;;;;

<|page-title@lavender:book_components|title=Note: Slot References|>Item slots may be referenced by spells.
Creating such a reference comes at no cost. However, using the reference in a way that moves the items within the slot, will incur a move cost.
This cost is equivalent to 32 + (distance * 0.8), per moved item. Slot references will always point to a block position, or use the *current caster at the time of move*.

;;;;;

<|glyph@trickster:templates|trick-id=trickster:other_hand_slot,title=Catch Delusion|>

-> slot

---

Returns a slot reference of the caster's other hand.

;;;;;

<|glyph@trickster:templates|trick-id=trickster:get_inventory_slot,title=Intrusive Ingress|>

number, [vector] -> slot

---

Returns the item slot at the given index in either the inventory of the caster, or the block at the given position.

;;;;;

<|glyph@trickster:templates|trick-id=trickster:check_hat,title=Cranium Delusion|>

-> number

---

Returns the selected slot in the caster's [Top Hat](^trickster:basics/top_hat).
