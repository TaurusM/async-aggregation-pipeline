## async-aggregation-pipeline

Just a small framework for crafting pipelines made for async aggregation of data

through an arbitrary of aggregating worker tasks, piped through also an arbitrary 

amount of filters. 

This is not what you want if your application is CPU throttled and/or you expect so much 

data that a single thread would not be able to keep up with it. 