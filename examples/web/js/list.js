import React from 'react';
import { proto, Registry, MutationBuilder, QueryBuilder, matchTrait } from 'exocore';

export default class List extends React.Component {
  constructor(props) {
    super(props);

    this.exocore = props.exocore;
    this.state = {entities: []};

    this.registerQuery();
  }

  render() {
    return (
      <div>
        <Input onAdd={this.onAdd.bind(this)}/>

        <ul>
          {this.renderList()}
        </ul>
      </div>
    );
  }

  renderList() {
    const DeleteButton = (props) => {
      return <button onClick={this.onDelete.bind(this, props.entity.id, props.trait.id)}>Delete</button>
    };

    return this.state.entities.map(res =>
      <li key={res.entity.id}>{res.trait.string1} (<DeleteButton entity={res.entity} trait={res.trait} />)</li>
    );
  }

  async onAdd(text) {
    const mutation = MutationBuilder
      .createEntity()
      .putTrait(new proto.exocore.test.TestMessage({
        string1: text,
      }))
      .build();

    await this.exocore.mutate(mutation);
  }

  async onDelete(entityId, traitId) {
    const mutation = MutationBuilder
      .updateEntity(entityId)
      .deleteTrait(traitId)
      .build();

    await this.exocore.mutate(mutation);
  }

  registerQuery() {
    const query = QueryBuilder
      .withTrait(proto.exocore.test.TestMessage)
      .count(100)
      .build();

    this.watched_query = this.exocore.watched_query(query);
    this.watched_query.on_change(() => {
      const results = proto.exocore.index.EntityResults.decode(this.watched_query.get());

      let res = results.entities.flatMap((res) => {
        return matchTrait(res.entity.traits[0], {
          [Registry.messageFullName(proto.exocore.test.TestMessage)]: (trait) => {
            return {entity: res.entity, trait: trait};
          }
        })
      });

      this.setState({
        entities: res
      })
    })
  }

  componentWillUnmount() {
    this.watched_query.free();
  }
}

class Input extends React.Component {
  constructor(props) {
    super(props);

    this.state = {
      text: ''
    }
  }

  render() {
    return (
      <div>
        <input value={this.state.text} onChange={this.onTextChange.bind(this)}/>
        <button onClick={this.onAddClick.bind(this)}>Add</button>
      </div>
    )
  }

  onTextChange(e) {
    this.setState({
      text: e.target.value
    });
  }

  onAddClick(e) {
    this.props.onAdd(this.state.text);
    this.setState({
      text: ''
    });
  }
}

