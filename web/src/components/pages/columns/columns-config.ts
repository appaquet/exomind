import _ from 'lodash';

export class ColumnsConfig {
    parts: Array<ColumnConfig> = [];

    constructor(parts: Array<ColumnConfig>) {
        this.parts = parts;
    }

    static fromString(strConfig: string): ColumnsConfig {
        const parts = _(strConfig.split('-'))
            .map((s: string) => ColumnsConfig.decodePart(s))
            .filter((i: string) => !_.isEmpty(i))
            .map((s: string) => new ColumnConfig(s))
            .value();
        return new ColumnsConfig(parts);
    }

    static forSearch(keywords: string): ColumnsConfig {
        return new ColumnsConfig([ColumnConfig.forSearch(keywords)]);
    }

    static forInbox(): ColumnsConfig {
        return new ColumnsConfig([ColumnConfig.forInbox()]);
    }

    static forSnoozed(): ColumnsConfig {
        return new ColumnsConfig([ColumnConfig.forSnoozed()]);
    }

    static forRecent(): ColumnsConfig {
        return new ColumnsConfig([ColumnConfig.forRecent()]);
    }

    static forCollections(): ColumnsConfig {
        return new ColumnsConfig([ColumnConfig.forCollections()]);
    }

    static forObject(objectId: string): ColumnsConfig {
        return new ColumnsConfig([ColumnConfig.forObject(objectId)]);
    }

    static forEntity(entityId: string): ColumnsConfig {
        return new ColumnsConfig([ColumnConfig.forEntity(entityId)]);
    }

    get empty(): boolean {
        return this.parts.length === 0;
    }

    set(col: number, value: ColumnConfig): ColumnsConfig {
        const ret = this.parts.slice(0, col + 1);
        ret[col] = value;
        return new ColumnsConfig(ret);
    }

    // Removes a column and its following
    unset(col: number): ColumnsConfig {
        const ret = this.parts.slice(0, col);
        return new ColumnsConfig(ret);
    }

    // Remove a column and keeps the following
    pop(col: number): ColumnsConfig {
        const ret = this.parts.slice(0, col).concat(this.parts.slice(col + 1));
        return new ColumnsConfig(ret);
    }

    toString(): string {
        return _(this.parts).map((s: ColumnConfig) => ColumnsConfig.encodePart(s.toString())).value().join('-');
    }

    static encodePart(string: string): string {
        return encodeURIComponent(string.toString().replace(/-/g, '%1%'));
    }

    static decodePart(string: string): string {
        return decodeURIComponent(string).replace(/%1%/g, '-');
    }
}

export class ColumnConfig {
    static InboxToken = 'i';
    static CollectionsToken = 'c';
    static SearchToken = 's';
    static ObjectToken = 'o';
    static EntityToken = 'e';
    static TraitToken = 't';
    static SnoozedToken = 'z';
    static RecentToken = 'r';

    part: string;
    splitPart: string[];

    constructor(part: string) {
        this.part = part;
        this.splitPart = part.slice(1).split(':');
    }

    static forSearch(keyword: string): ColumnConfig {
        return new ColumnConfig(ColumnConfig.SearchToken + keyword);
    }

    static forInbox(): ColumnConfig {
        return new ColumnConfig(ColumnConfig.InboxToken);
    }

    static forSnoozed(): ColumnConfig {
        return new ColumnConfig(ColumnConfig.SnoozedToken);
    }

    static forRecent(): ColumnConfig {
        return new ColumnConfig(ColumnConfig.RecentToken);
    }

    static forCollections(): ColumnConfig {
        return new ColumnConfig(ColumnConfig.CollectionsToken);
    }

    static forObject(objectId: string): ColumnConfig {
        return new ColumnConfig(ColumnConfig.ObjectToken + objectId);
    }

    static forEntity(entityId: string): ColumnConfig {
        return new ColumnConfig(ColumnConfig.EntityToken + entityId);
    }

    static forTrait(entityId: string, traitId: string): ColumnConfig {
        return new ColumnConfig(ColumnConfig.TraitToken + entityId).withExtra(traitId);
    }

    toString(): string {
        return this.part;
    }

    get token(): string {
        return this.part.slice(0, 1);
    }

    get isInbox(): boolean {
        return this.token === ColumnConfig.InboxToken;
    }

    get isCollection(): boolean {
        return this.token === ColumnConfig.CollectionsToken;
    }

    get isSearch(): boolean {
        return this.token === ColumnConfig.SearchToken;
    }

    get isObject(): boolean {
        return this.token === ColumnConfig.ObjectToken;
    }

    get isEntity(): boolean {
        return this.token === ColumnConfig.EntityToken;
    }

    get isTrait(): boolean {
        return this.token === ColumnConfig.TraitToken;
    }

    get isSnoozed(): boolean {
        return this.token === ColumnConfig.SnoozedToken;
    }

    get isHistory(): boolean {
        return this.token === ColumnConfig.RecentToken;
    }

    get value(): string {
        return this.splitPart[0];
    }

    get extra(): string | null {
        return (this.splitPart.length > 1) ? this.splitPart[1] : null;
    }

    withExtra(extra: string): ColumnConfig {
        const p = [this.token + this.value];
        if (!_.isNull(extra)) {
            p.push(extra);
        }
        return new ColumnConfig(p.join(':'));
    }

}
