# Multiple Pages

Sometimes, you need to have multiple pages spawned at the same time, or you need to have one of the pages deactivated
and
the other activated (e.g. for dialog/modal systems).

You can do all of this by using the `PageManager` resource.

## Spawning Pages

As we learned in previous sections, you can spawn pages into the world using `PageManager::spawn`. You can also despawn
them using `PageManager::despawn`.

When pages are spawned, they are deactivated by default. You can activate/deactivate them via `PageManager::set_active`.

## Activating Pages

Deactivated pages are not only hidden from the user, but they can also not be interacted with. Elements inside an
activated page have the `ElementActive` component. You can use this component to filter queries (like
`With<ElementActive>`) for active elements.

Activating and deactivating pages also trigger `ActivatePage` and `DeactivatePage` events, respectively.

**NOTE:** Deactivated pages are still present in the world and the memory. You should despawn pages if you don't need
them for a longer time.

If you want to look at an example of how to use multiple pages and page (de)activation, you can check out
the [multi_page](../examples/multi_page.rs) example.
