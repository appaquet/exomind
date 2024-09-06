
import classNames from 'classnames';
import { exocore, Exocore, MutationBuilder, QueryBuilder, TraitQueryBuilder, WatchedQueryWrapper } from 'exocore';
import { exomind } from '../../../protos';
import React from 'react';
import Navigation from '../../../navigation';
import { EntityTrait, EntityTraits, TraitIcon } from '../../../utils/entities';
import Path from '../../../utils/path';
import './hamburger.less';
import EntityIcon from '../../objects/entity-icon';
import DragAndDrop, { DragData } from '../../interaction/drag-and-drop/drag-and-drop';
import { observer } from 'mobx-react';
import { IStores, StoresContext } from '../../../stores/stores';
import { getEntityParentRelation } from '../../../stores/collections';

interface IProps {
  path: Path;
}

interface IState {
  favorites: EntityTraits[];
}

@observer
export default class Hamburger extends React.Component<IProps, IState> {
  static contextType = StoresContext;
  declare context: IStores;

  private favoritesQuery: WatchedQueryWrapper;

  constructor(props: IProps) {
    super(props);

    const traitQuery = TraitQueryBuilder.refersTo('collection', 'favorites').build();
    const childrenQuery = QueryBuilder
      .withTrait(exomind.base.v1.CollectionChild, traitQuery)
      .count(30)
      .orderByField('weight', false)
      .build();

    this.favoritesQuery = Exocore.store.watchedQuery(childrenQuery);
    this.favoritesQuery.onChange((res) => {
      this.setState({
        favorites: res.entities.map((entity) => new EntityTraits(entity.entity)),
      });
    });

    this.state = {
      favorites: [],
    };
  }

  componentWillUnmount(): void {
    this.favoritesQuery.free();
  }

  render(): React.ReactNode {
    const classes = classNames({
      'open': true,
    });

    const inbox = this.context.collections.getCollection('inbox');

    return (
      <div id="hamburger" className={classes}>
        <ul>
          <HamburgerLink path={this.props.path} link={Navigation.pathForInbox()} label="Inbox" icon={{ fa: "inbox" }} trait={inbox} />
          <li className="sep" key={'inbox_sep'} />

          <HamburgerLink path={this.props.path} link={Navigation.pathForSnoozed()} label="Snoozed" icon={{ fa: "clock-o" }} />
          <li className="sep" key={'snoozed_sep'} />

          <HamburgerLink path={this.props.path} link={Navigation.pathForRecent()} label="Recent" icon={{ fa: "history" }} />
          <li className="sep" key={'recent_sep'} />

          {this.renderFavorites()}

          <HamburgerLink path={this.props.path} link={Navigation.pathForSettings()} label="Settings" icon={{ fa: "cog" }} />
        </ul>
      </div>
    );
  }

  private renderFavorites(): React.ReactNode {
    return this.state.favorites
      .flatMap((entity) => {
        const priorityTrait = entity.priorityTrait;

        return [
          <HamburgerLink
            path={this.props.path}
            key={entity.id}
            link={Navigation.pathForEntity(entity)}
            label={priorityTrait.displayName}
            trait={priorityTrait}
            icon={priorityTrait.icon} />,

          <li className="sep" key={entity.id + '_sep'} />
        ];
      });
  }
}

const HamburgerLink = (props: { path: Path, link: string, label: string, icon?: TraitIcon, trait?: EntityTrait<unknown> }) => {
  const classes = classNames({
    active: ('/' + props.path.toString()).startsWith(props.link)
  });

  const droppable = props.trait?.constants.collectionLike ?? false;

  return (
    <li className={classes}>
      <DragAndDrop
        draggable={false}
        droppable={droppable}
        dropPositions={['into']}
        onDropIn={(data) => handleDropIn(props.trait, data)}
      >
        <a href={props.link}>
          <EntityIcon icon={props.icon} trait={props.trait} />

          <span className="text">{props.label}</span>
        </a>
      </DragAndDrop>
    </li>
  );
};

function handleDropIn(intoEntity: EntityTrait<unknown>, data: DragData) {
  const droppedEntity = data.object as EntityTraits;
  const parentId = intoEntity.et.id;
  const droppedEntityRelation = getEntityParentRelation(intoEntity.et, parentId);
  const relationTraitId = droppedEntityRelation?.id ?? `child_${parentId}`;

  let mb = MutationBuilder
    .updateEntity(droppedEntity.id)
    .putTrait(new exomind.base.v1.CollectionChild({
      collection: new exocore.store.Reference({
        entityId: parentId
      }),
      weight: new Date().getTime(),
    }), relationTraitId)
    .returnEntities();

  if (!!data.parentObject && data.parentObject instanceof EntityTraits) {
    // if it has been moved and it's not inside its own container, then we remove it from old parent
    const fromParentEntity = data.parentObject;
    if (data.effect === 'move' && !!fromParentEntity && fromParentEntity && parentId !== fromParentEntity.id) {
      const fromRelation = getEntityParentRelation(droppedEntity, fromParentEntity.id);
      mb = mb.deleteTrait(fromRelation.id);
    }
  }

  Exocore.store.mutate(mb.build());
}