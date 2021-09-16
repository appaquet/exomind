
import React from 'react';
import Navigation from '../../../navigation';
import Debouncer from '../../../utils/debouncer';
import Path from '../../../utils/path';
import { ColumnConfigs } from '../../pages/columns/columns-config';
import './header.less';
import { StoresContext, IStores } from '../../../stores/stores';

interface IProps {
  path: Path;
}

interface IState {
  pathKeywords?: string;
  fieldKeywords: string;
  debouncedKeywords: string;
}

export class Header extends React.Component<IProps, IState> {
  static contextType = StoresContext;
  declare context: IStores;

  private debouncer: Debouncer;

  constructor(props: IProps) {
    super(props);

    this.debouncer = new Debouncer(50);

    const keywords = this.keywordsFromPath(props);
    this.state = {
      pathKeywords: keywords,
      fieldKeywords: keywords,
      debouncedKeywords: keywords,
    };
  }

  componentDidUpdate(prevProps: IProps): void {
    const previousPathKeywords = this.keywordsFromPath(prevProps);
    const pathKeywords = this.keywordsFromPath(this.props);

    // nothing to do if keywords didn't change in URL since last props
    if (previousPathKeywords == pathKeywords) {
      return;
    }

    // keywords from path just got updated from latest debounced keywords
    if (pathKeywords == this.state.debouncedKeywords) {
      this.setState({
        pathKeywords: pathKeywords,
      });
      return;
    }

    // keywords from path have an unexpected value, so we reset the local state
    this.setState({
      pathKeywords: pathKeywords,
      fieldKeywords: pathKeywords,
      debouncedKeywords: pathKeywords,
    });
  }

  render(): React.ReactNode {
    return (
      <nav id="header" className="navbar navbar-fixed-top">
        <div className="container-fluid">
          {this.context.session.cellInitialized ? this.renderSearchbox() : undefined}
        </div>
      </nav>
    );
  }

  private renderSearchbox() {
    return (
      <div className="search-col form-group">
        <div className="input-group">
          <span className="glyphicon glyphicon-search input-group-addon icon" />
          <input type="text" className="form-control" value={this.state.fieldKeywords} onChange={(event) => this.handleSearchChange(event)} />
        </div>
      </div>
    );
  }

  private handleSearchChange(event: React.ChangeEvent<HTMLInputElement>) {
    const keywords = event.target.value;

    this.setState({
      fieldKeywords: keywords,
    });

    this.debouncer.debounce(() => {
      this.setState({
        debouncedKeywords: keywords,
      });

      // when current in search, we replace state, otherwise we push it
      const replace = !!this.state.pathKeywords;
      Navigation.navigate(Navigation.pathForSearch(keywords), replace);
    });
  }

  private keywordsFromPath(props: IProps): string {
    let keyword = '';
    if (Navigation.isColumnsPath(props.path)) {
      const config = ColumnConfigs.fromString(props.path.drop(1).toString());
      if (!config.empty && config.parts[0].isSearch) {
        keyword = config.parts[0].first;
      }
    }

    return keyword
  }
}

