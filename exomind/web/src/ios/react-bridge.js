
import React from 'react';
import IosHtmlEditor from './ios-html-editor/ios-html-editor';

export default class ReactBridge extends React.Component {
  constructor(props) {
    super(props);

    window.setData = (data) => {
      this.setState({ data: data });
    };
    this.state = { component: window.component, data: null };
  }

  render() {
    if (this.state.component === 'html-editor') {
      return <IosHtmlEditor {...this.state.data} />;
    } else {
      return <div>Unknown component {this.state.component}</div>;
    }
  }
}

