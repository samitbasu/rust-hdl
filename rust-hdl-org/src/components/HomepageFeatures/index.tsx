import React from 'react';
import clsx from 'clsx';
import styles from './styles.module.css';

type FeatureItem = {
  title: string;
  Svg: React.ComponentType<React.ComponentProps<'svg'>>;
  description: JSX.Element;
};

const FeatureList: FeatureItem[] = [
  {
    title: 'Safe',
    Svg: require('@site/static/img/cyber_ferris_safe.svg').default,
    description: (
      <>
        Use <code>rustc</code> to check the validity of your firmware with
        strongly typed interfaces that are checked at <i>compile</i> time.
      </>
    ),
  },
  {
    title: 'Powerful',
    Svg: require('@site/static/img/cyber_ferris_powerful.svg').default,
    description: (
      <>
        Easily package complex designs into easy-to-reuse modules that
        you can reuse easily.  Connecting components is simple and missing or erroneous
        connections are caught at compile time or with the built in static analysis passes.
      </>
    ),
  },
  {
    title: 'Batteries Included',
    Svg: require('@site/static/img/cyber_ferris_batteries.svg').default,
    description: (
      <>
        Need an asynchronous FIFO?  Or a SDR memory controller?  Or a one shot?
        Use the provided set of <i>widgets</i> to get started.  Most are generic
        and can be used to handle arbitrary data types.
      </>
    ),
  },
];

function Feature({ title, Svg, description }: FeatureItem) {
  return (
    <div className={clsx('col col--4')}>
      <div className="text--center">
        <Svg className={styles.featureSvg} role="img" />
      </div>
      <div className="text--center padding-horiz--md">
        <h3>{title}</h3>
        <p>{description}</p>
      </div>
    </div>
  );
}

export default function HomepageFeatures(): JSX.Element {
  return (
    <section className={styles.features}>
      <div className="container">
        <div className="row">
          {FeatureList.map((props, idx) => (
            <Feature key={idx} {...props} />
          ))}
        </div>
      </div>
    </section>
  );
}
