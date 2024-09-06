import _ from 'lodash';

export class ColumnConfigs {
    parts: Array<ColumnConfig> = [];

    constructor(parts: ColumnConfig[] = []) {
        this.parts = parts;
    }

    static fromString(strConfig: string): ColumnConfigs {
        const parts = _(strConfig.split('-'))
            .map((s: string) => ColumnConfigs.decodePart(s))
            .filter((i: string) => !_.isEmpty(i))
            .map((s: string) => ColumnConfig.fromString(s))
            .value();
        return new ColumnConfigs(parts);
    }

    static forSearch(keywords: string): ColumnConfigs {
        return new ColumnConfigs([ColumnConfig.forSearch(keywords)]);
    }

    static forInbox(): ColumnConfigs {
        return new ColumnConfigs([ColumnConfig.forInbox()]);
    }

    static forSnoozed(): ColumnConfigs {
        return new ColumnConfigs([ColumnConfig.forSnoozed()]);
    }

    static forRecent(): ColumnConfigs {
        return new ColumnConfigs([ColumnConfig.forRecent()]);
    }

    static forCollections(): ColumnConfigs {
        return new ColumnConfigs([ColumnConfig.forCollections()]);
    }

    static forObject(objectId: string): ColumnConfigs {
        return new ColumnConfigs([ColumnConfig.forObject(objectId)]);
    }

    static forEntity(entityId: string): ColumnConfigs {
        return new ColumnConfigs([ColumnConfig.forEntity(entityId)]);
    }

    get empty(): boolean {
        return this.parts.length === 0;
    }

    set(col: number, value: ColumnConfig): ColumnConfigs {
        const ret = this.parts.slice(0, col + 1);
        ret[col] = value;
        return new ColumnConfigs(ret);
    }

    // Removes a column and its following
    unset(col: number): ColumnConfigs {
        const ret = this.parts.slice(0, col);
        return new ColumnConfigs(ret);
    }

    // Remove a column and keeps the following
    pop(col: number): ColumnConfigs {
        const ret = this.parts.slice(0, col).concat(this.parts.slice(col + 1));
        return new ColumnConfigs(ret);
    }

    get last(): ColumnConfig | null {
        return this.parts.length > 0 ? this.parts[this.parts.length - 1] : null;
    }

    toString(): string {
        return _(this.parts).map((s: ColumnConfig) => ColumnConfigs.encodePart(s.toString())).value().join('-');
    }

    static encodePart(string: string): string {
        return encodeURIComponent(string.toString().replace(/-/g, '%1%'));
    }

    static decodePart(string: string): string {
        return decodeURIComponent(string).replace(/%1%/g, '-');
    }

    equals(other: ColumnConfigs): boolean {
        return this.toString() === other.toString();
    }
}

export const InboxToken = 'i';
export const CollectionsToken = 'c';
export const SearchToken = 's';
export const ObjectToken = 'o';
export const EntityToken = 'e';
export const TraitToken = 't';
export const SnoozedToken = 'z';
export const RecentToken = 'r';
export const MultipleToken = 'm';

export type ColumnType = 'i' | 'c' | 's' | 'o' | 'e' | 't' | 'z' | 'r' | 'm';

const PartDelim = '::';

export class ColumnConfig {
    public type: ColumnType;
    public parts: string[];

    constructor(type: ColumnType, parts?: string | string[]) {
        this.type = type;

        if (typeof parts == 'string') {
            this.parts = parts.split(PartDelim);
        } else if (Array.isArray(parts)) {
            this.parts = parts;
        } else {
            this.parts = [];
        }
    }

    static fromString(value: string): ColumnConfig {
        return new ColumnConfig(value[0] as ColumnType, value.slice(1));
    }

    toString(): string {
        return this.type + this.parts.join(PartDelim);
    }

    static forSearch(keyword: string): ColumnConfig {
        return new ColumnConfig(SearchToken, keyword);
    }

    static forInbox(): ColumnConfig {
        return new ColumnConfig(InboxToken);
    }

    static forSnoozed(): ColumnConfig {
        return new ColumnConfig(SnoozedToken);
    }

    static forRecent(): ColumnConfig {
        return new ColumnConfig(RecentToken);
    }

    static forCollections(): ColumnConfig {
        return new ColumnConfig(CollectionsToken);
    }

    static forObject(objectId: string): ColumnConfig {
        return new ColumnConfig(ObjectToken, objectId);
    }

    static forEntity(entityId: string): ColumnConfig {
        return new ColumnConfig(EntityToken, entityId);
    }

    static forTrait(entityId: string, traitId: string): ColumnConfig {
        return new ColumnConfig(TraitToken, entityId).withExtra(traitId);
    }

    static forMultiple(configs: ColumnConfig[]): ColumnConfig {
        const inner = configs.map((config) => config.toString()).join(PartDelim);
        return new ColumnConfig(MultipleToken, inner);
    }

    get isInbox(): boolean {
        return this.type === InboxToken;
    }

    get isCollection(): boolean {
        return this.type === CollectionsToken;
    }

    get isSearch(): boolean {
        return this.type === SearchToken;
    }

    get isObject(): boolean {
        return this.type === ObjectToken;
    }

    get isEntity(): boolean {
        return this.type === EntityToken;
    }

    get isTrait(): boolean {
        return this.type === TraitToken;
    }

    get isSnoozed(): boolean {
        return this.type === SnoozedToken;
    }

    get isRecent(): boolean {
        return this.type === RecentToken;
    }

    get isMultiple(): boolean {
        return this.type === MultipleToken;
    }

    get first(): string {
        return this.parts[0];
    }

    get second(): string | null {
        return (this.parts.length > 1) ? this.parts[1] : null;
    }

    get values(): ColumnConfig[] {
        return Array.from(this.parts.map((value) => ColumnConfig.fromString(value)));
    }

    withExtra(part: string): ColumnConfig {
        const newParts = Array.from(this.parts);
        if (!_.isNull(part)) {
            newParts.push(part);
        }
        return new ColumnConfig(this.type, newParts.join(PartDelim));
    }
}
