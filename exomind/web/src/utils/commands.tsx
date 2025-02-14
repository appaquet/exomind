import { exocore, Exocore, MutationBuilder } from "exocore";
import { EntityTrait, EntityTraits } from "./entities";
import { exomind } from "../protos";
import { PINNED_WEIGHT, WEIGHT_SPACING } from "../stores/collections";
import { toProtoTimestamp } from "./dates";

export class Commands {
    static async removeFromParent(entities: EntityTraits | EntityTraits[], parent: EntityTraits | string) {
        if (!Array.isArray(entities)) {
            entities = [entities];
        }

        const promises = [];
        const parentId = this.getEntityId(parent);
        for (const et of entities) {
            const mb = MutationBuilder.updateEntity(et.entity.id);
            const parentRelation = this.getEntityParentRelation(et, parentId);
            if (parentRelation) {
                mb.deleteTrait(parentRelation.id);
                promises.push(Exocore.store.mutate(mb.build()));
            }
        }

        return await Promise.all(promises);
    }

    static async addToParent(entities: EntityTraits | EntityTraits[], parent: EntityTraits | string, pinned = false) {
        if (!Array.isArray(entities)) {
            entities = [entities];
        }

        const parentId = this.getEntityId(parent);

        let weight = new Date().getTime() - (entities.length - 1 * WEIGHT_SPACING);
        if (pinned) {
            weight += PINNED_WEIGHT;
        }

        const promises = [];
        for (const et of entities) {
            const mb = MutationBuilder.updateEntity(et.entity.id);

            const parentRelation = this.getEntityParentRelation(et, parentId);
            if (parentRelation) {
                parentRelation.message.weight = weight;
                mb.putTrait(parentRelation.message, parentRelation.id);
            } else {
                mb.putTrait(new exomind.base.v1.CollectionChild({
                    collection: new exocore.store.Reference({
                        entityId: parentId,
                    }),
                    weight: new Date().getTime(),
                }), `child_${parentId}`);
            }

            promises.push(Exocore.store.mutate(mb.build()));

            weight += WEIGHT_SPACING;
        }

        return await Promise.all(promises);
    }

    static async pinEntityInParent(entities: EntityTraits | EntityTraits[], parent: EntityTraits | string) {
        return this.addToParent(entities, parent, true);
    }

    static async unpinEntityInParent(entities: EntityTraits | EntityTraits[], parent: EntityTraits | string) {
        return this.addToParent(entities, parent, false);
    }

    static async snooze(entities: EntityTraits | EntityTraits[], date: Date, parent: EntityTraits | string | null = null, removeFromParent = true) {
        if (!Array.isArray(entities)) {
            entities = [entities];
        }

        const parentId = this.getEntityId(parent);

        const promises = [];
        for (const et of entities) {
            let mb = MutationBuilder
                .updateEntity(et.id)
                .putTrait(new exomind.base.v1.Snoozed({
                    untilDate: toProtoTimestamp(date),
                }), "snoozed");

            if (removeFromParent) {
                const parentRel = this.getEntityParentRelation(et, parentId);
                if (parentRel) {
                    mb = mb.deleteTrait(parentRel.id);
                }
            }

            promises.push(Exocore.store.mutate(mb.build()));
        }

        return await Promise.all(promises);
    }

    static async removeSnooze(entities: EntityTraits | EntityTraits[]) {
        if (!Array.isArray(entities)) {
            entities = [entities];
        }

        const promises = [];
        for (const et of entities) {
            const snooze = et.traitOfType<exomind.base.v1.Snoozed>(exomind.base.v1.Snoozed);
            if (snooze) {
                const mb = MutationBuilder
                    .updateEntity(et.id)
                    .deleteTrait(snooze.id);

                promises.push(Exocore.store.mutate(mb.build()));
            }
        }

        return await Promise.all(promises);
    }

    static async delete(entities: EntityTraits | EntityTraits[]) {
        if (!Array.isArray(entities)) {
            entities = [entities];
        }

        const promises = [];
        for (const et of entities) {
            promises.push(Exocore.store.mutate(MutationBuilder.deleteEntity(et.id)));
        }

        return await Promise.all(promises);
    }

    static async createNote(parent: EntityTraits | string, title: string | null = null): Promise<IEntityCreateResult> {
        const parentId = this.getEntityId(parent);

        const mutation = MutationBuilder
            .createEntity()
            .putTrait(new exomind.base.v1.Note({
                title: title || 'New note',
            }))
            .putTrait(new exomind.base.v1.CollectionChild({
                collection: new exocore.store.Reference({
                    entityId: parentId,
                }),
                weight: new Date().getTime(),
            }), `child_${parentId}`)
            .returnEntities()
            .build();

        return await this.executeNewEntityMutation(mutation);
    }

    static async createTask(parent: EntityTraits | string, title: string | null = null): Promise<IEntityCreateResult> {
        const parentId = this.getEntityId(parent);

        const mutation = MutationBuilder
            .createEntity()
            .putTrait(new exomind.base.v1.Task({
                title: title || 'New task',
            }))
            .putTrait(new exomind.base.v1.CollectionChild({
                collection: new exocore.store.Reference({
                    entityId: parentId,
                }),
                weight: new Date().getTime(),
            }), `child_${parentId}`)
            .returnEntities()
            .build();

        return await this.executeNewEntityMutation(mutation);
    }

    static async createCollection(parent: EntityTraits | string, name: string | null = null): Promise<IEntityCreateResult> {
        const parentId = this.getEntityId(parent);

        const mutation = MutationBuilder
            .createEntity()
            .putTrait(new exomind.base.v1.Collection({
                name: name || 'New collection',
            }))
            .putTrait(new exomind.base.v1.CollectionChild({
                collection: new exocore.store.Reference({
                    entityId: parentId,
                }),
                weight: new Date().getTime(),
            }), `child_${parentId}`)
            .returnEntities()
            .build();

        return await this.executeNewEntityMutation(mutation);
    }

    static async createLink(parent: EntityTraits | string, url: string, title: string | null = null): Promise<IEntityCreateResult> {
        const parentId = this.getEntityId(parent);

        const mutation = MutationBuilder
            .createEntity()
            .putTrait(new exomind.base.v1.Link({
                url: url,
                title: title || url,
            }))
            .putTrait(new exomind.base.v1.CollectionChild({
                collection: new exocore.store.Reference({
                    entityId: parentId,
                }),
                weight: new Date().getTime(),
            }), `child_${parentId}`)
            .returnEntities()
            .build();

        return await this.executeNewEntityMutation(mutation);
    }

    static getEntityParentRelation(entity: EntityTraits, parentId: string): EntityTrait<exomind.base.v1.CollectionChild> {
        return entity
            .traitsOfType<exomind.base.v1.CollectionChild>(exomind.base.v1.CollectionChild)
            .filter((e) => e.message.collection.entityId == parentId)
            .shift();
    }

    static getEntityId(entity: EntityTraits | string | null): string | null {
        if (!entity) {
            return null;
        }

        if (typeof entity === 'string') {
            return entity;
        }

        return entity.id;
    }

    private static async executeNewEntityMutation(mutation: exocore.store.MutationRequest): Promise<IEntityCreateResult> {
        const result = await Exocore.store.mutate(mutation);
        if (result.entities.length > 0) {
            return {
                entity: new EntityTraits(result.entities[0]),
            };
        } else {
            return {
                error: "no entity returned",
            };
        }
    }
}

export interface IEntityCreateResult {
    entity?: EntityTraits;
    error?: string;
}