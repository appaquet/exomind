
import classNames from 'classnames';
import { Exocore, QueryBuilder, TraitQueryBuilder, WatchedQueryWrapper } from 'exocore';
import { exomind } from '../../../protos';
import React from 'react';
import Navigation from '../../../navigation';
import { EntityTrait, EntityTraits, TraitIcon } from '../../../store/entities';
import Path from '../../../utils/path';
import './hamburger.less';
import EntityIcon from '../../objects/entity-icon';

interface IProps {
  path: Path;
}

interface IState {
  favorites: EntityTraits[];
}

export default class Hamburger extends React.Component<IProps, IState> {
  private favoritesQuery: WatchedQueryWrapper;

  constructor(props: IProps) {
    super(props);

    const traitQuery = TraitQueryBuilder.refersTo('collection', 'favorites').build();
    const childrenQuery = QueryBuilder
      .withTrait(exomind.base.CollectionChild, traitQuery)
      .count(30)
      .orderByField('weight', false)
      .build();

    this.favoritesQuery = Exocore.store.watchedQuery(childrenQuery);
    this.favoritesQuery.onChange((res) => {
      this.setState({
        favorites: res.entities.map((entity) => new EntityTraits(entity.entity)),
      })
    });

    this.state = {
      favorites: [],
    }
  }

  componentWillUnmount(): void {
    this.favoritesQuery.free();
  }

  render(): React.ReactNode {
    const classes = classNames({
      'open': true,
    });

    return (
      <div id="hamburger" className={classes}>
        <ul>
          <HamburgerLink path={this.props.path} link={Navigation.pathForInbox()} label="Inbox" icon={{ fa: "inbox" }} />
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

  return (
    <li className={classes}>
      <a href={props.link}>
        <EntityIcon icon={props.icon} trait={props.trait} />

        <span className="text">{props.label}</span>
      </a>
    </li>
  );
}

