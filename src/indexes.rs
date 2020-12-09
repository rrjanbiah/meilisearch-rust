use crate::{
    client::Client, document::*, errors::Error, progress::*, request::*, search::*,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::json;
use std::{fmt::Display, collections::HashMap};

#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
pub(crate) struct JsonIndex {
    uid: String,
    primaryKey: Option<String>,
    createdAt: String,
    updatedAt: String,
}

impl JsonIndex {
    pub(crate) fn into_index<'a>(self, client: &'a Client) -> Index<'a> {
        Index {
            uid: self.uid,
            client,
        }
    }
}

/// An index containing [Documents](../document/trait.Document.html).
///
/// # Example
///
/// ```
/// # use meilisearch_sdk::{client::*, indexes::*};
/// # #[tokio::main]
/// # async fn main() {
/// let client = Client::new("http://localhost:7700", "masterKey");
///
/// // get the index called movies or create it if it does not exist
/// let movies = client.get_or_create("movies").await.unwrap();
///
/// // do something with the index
/// # }
/// ```
#[derive(Debug)]
pub struct Index<'a> {
    pub(crate) uid: String,
    pub(crate) client: &'a Client<'a>,
}

impl<'a> Index<'a> {
    /// Set the primary key of the index.
    ///
    /// If you prefer, you can use the method [set_primary_key](#method.set_primary_key), which is an alias.
    pub async fn update(&self, primary_key: &str) -> Result<(), Error> {
        request::<serde_json::Value, JsonIndex>(
            &format!("{}/indexes/{}", self.client.host, self.uid),
            self.client.apikey,
            Method::Put(json!({ "primaryKey": primary_key })),
            200,
        ).await?;
        Ok(())
    }

    /// Delete the index.
    ///
    /// # Example
    ///
    /// ```
    /// # use meilisearch_sdk::{client::*, indexes::*};
    /// # #[tokio::main]
    /// # async fn main() {
    /// let client = Client::new("http://localhost:7700", "masterKey");
    /// # client.create_index("movies", None).await;
    ///
    /// // get the index named "movies" and delete it
    /// let movies = client.get_index("movies").await.unwrap();
    /// movies.delete().await.unwrap();
    /// # }
    /// ```
    pub async fn delete(self) -> Result<(), Error> {
        Ok(request::<(), ()>(
            &format!("{}/indexes/{}", self.client.host, self.uid),
            self.client.apikey,
            Method::Delete,
            204,
        ).await?)
    }

    /// Search for documents matching a specific query in the index.\
    /// See also the [search method](#method.search).
    ///
    /// # Example
    ///
    /// ```
    /// use serde::{Serialize, Deserialize};
    /// # use meilisearch_sdk::{client::*, indexes::*, document::*, search::*};
    ///
    /// #[derive(Serialize, Deserialize, Debug)]
    /// struct Movie {
    ///     name: String,
    ///     description: String,
    /// }
    /// // that trait is used by the sdk when the primary key is needed
    /// impl Document for Movie {
    ///     type UIDType = String;
    ///     fn get_uid(&self) -> &Self::UIDType {
    ///         &self.name
    ///     }
    /// }
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// let client = Client::new("http://localhost:7700", "masterKey");
    /// let mut movies = client.get_or_create("movies").await.unwrap();
    ///
    /// // add some documents
    /// # movies.add_or_replace(&[Movie{name:String::from("Interstellar"), description:String::from("Interstellar chronicles the adventures of a group of explorers who make use of a newly discovered wormhole to surpass the limitations on human space travel and conquer the vast distances involved in an interstellar voyage.")},Movie{name:String::from("Unknown"), description:String::from("Unknown")}], Some("name")).await.unwrap();
    /// # std::thread::sleep(std::time::Duration::from_secs(1));
    ///
    /// let query = Query::new(&movies).with_query("Interstellar").with_limit(5).build();
    /// let results = movies.execute_query::<Movie>(&query).await.unwrap();
    /// # assert!(results.hits.len()>0);
    /// # }
    /// ```
    pub async fn execute_query<T: 'static + DeserializeOwned>(
        &self,
        query: &Query<'_>,
    ) -> Result<SearchResults<T>, Error> {
        Ok(request::<&Query, SearchResults<T>>(
            &format!(
                "{}/indexes/{}/search",
                self.client.host,
                self.uid
            ),
            self.client.apikey,
            Method::Post(query),
            200,
        ).await?)
    }

    /// Search for documents matching a specific query in the index.\
    /// See also the [execute_query method](#method.execute_query).
    ///
    /// # Example
    ///
    /// ```
    /// use serde::{Serialize, Deserialize};
    /// # use meilisearch_sdk::{client::*, indexes::*, document::*, search::*};
    ///
    /// #[derive(Serialize, Deserialize, Debug)]
    /// struct Movie {
    ///     name: String,
    ///     description: String,
    /// }
    /// // that trait is used by the sdk when the primary key is needed
    /// impl Document for Movie {
    ///     type UIDType = String;
    ///     fn get_uid(&self) -> &Self::UIDType {
    ///         &self.name
    ///     }
    /// }
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// let client = Client::new("http://localhost:7700", "masterKey");
    /// # client.delete_index("movies").await;
    /// let mut movies = client.get_or_create("movies").await.unwrap();
    ///
    /// // add some documents
    /// # movies.add_or_replace(&[Movie{name:String::from("Interstellar"), description:String::from("Interstellar chronicles the adventures of a group of explorers who make use of a newly discovered wormhole to surpass the limitations on human space travel and conquer the vast distances involved in an interstellar voyage.")},Movie{name:String::from("Unknown"), description:String::from("Unknown")}], Some("name")).await.unwrap();
    /// # std::thread::sleep(std::time::Duration::from_secs(1));
    ///
    /// let results = movies.search()
    ///     .with_query("Interstellar")
    ///     .with_limit(5)
    ///     .execute::<Movie>()
    ///     .await
    ///     .unwrap();
    /// # assert!(results.hits.len()>0);
    /// # }
    /// ```
    pub fn search(&self) -> Query {
        Query::new(self)
    }

    /// Get one [document](../document/trait.Document.html) using its unique id.
    /// Serde is needed. Add `serde = {version="1.0", features=["derive"]}` in the dependencies section of your Cargo.toml.
    ///
    /// # Example
    ///
    /// ```
    /// use serde::{Serialize, Deserialize};
    ///
    /// # use meilisearch_sdk::{client::*, indexes::*, document::*};
    ///
    /// #[derive(Serialize, Deserialize, Debug)]
    /// # #[derive(PartialEq)]
    /// struct Movie {
    ///    name: String,
    ///    description: String,
    /// }
    ///
    /// // that trait is used by the sdk when the primary key is needed
    /// impl Document for Movie {
    ///    type UIDType = String;
    ///    fn get_uid(&self) -> &Self::UIDType {
    ///        &self.name
    ///    }
    /// }
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// let client = Client::new("http://localhost:7700", "masterKey");
    /// # client.create_index("movies", None).await;
    /// let movies = client.get_index("movies").await.unwrap();
    /// # let mut movies = client.get_index("movies").await.unwrap();
    /// # movies.add_or_replace(&[Movie{name:String::from("Interstellar"), description:String::from("Interstellar chronicles the adventures of a group of explorers who make use of a newly discovered wormhole to surpass the limitations on human space travel and conquer the vast distances involved in an interstellar voyage.")}], Some("name")).await.unwrap();
    /// # std::thread::sleep(std::time::Duration::from_secs(1));
    /// #
    /// // retrieve a document (you have to put the document in the index before)
    /// let interstellar = movies.get_document::<Movie>(String::from("Interstellar")).await.unwrap();
    ///
    /// assert_eq!(interstellar, Movie{
    ///     name: String::from("Interstellar"),
    ///     description: String::from("Interstellar chronicles the adventures of a group of explorers who make use of a newly discovered wormhole to surpass the limitations on human space travel and conquer the vast distances involved in an interstellar voyage.")
    /// });
    /// # }
    /// ```
    pub async fn get_document<T: 'static + Document>(&self, uid: T::UIDType) -> Result<T, Error> {
        Ok(request::<(), T>(
            &format!(
                "{}/indexes/{}/documents/{}",
                self.client.host, self.uid, uid
            ),
            self.client.apikey,
            Method::Get,
            200,
        ).await?)
    }

    /// Get [documents](../document/trait.Document.html) by batch.
    ///
    /// Using the optional parameters offset and limit, you can browse through all your documents.
    /// If None, offset will be set to 0, limit to 20, and all attributes will be retrieved.
    ///
    /// *Note: Documents are ordered by MeiliSearch depending on the hash of their id.*
    ///
    /// # Example
    ///
    /// ```
    /// use serde::{Serialize, Deserialize};
    ///
    /// # use meilisearch_sdk::{client::*, indexes::*, document::*};
    ///
    /// #[derive(Serialize, Deserialize, Debug)]
    /// # #[derive(PartialEq)]
    /// struct Movie {
    ///    name: String,
    ///    description: String,
    /// }
    ///
    /// // that trait is used by the sdk when the primary key is needed
    /// impl Document for Movie {
    ///    type UIDType = String;
    ///    fn get_uid(&self) -> &Self::UIDType {
    ///        &self.name
    ///    }
    /// }
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// let client = Client::new("http://localhost:7700", "masterKey");
    /// # client.create_index("movies", None).await;
    /// let movie_index = client.get_index("movies").await.unwrap();
    /// # let mut movie_index = client.get_index("movies").await.unwrap();
    ///
    /// # movie_index.add_or_replace(&[Movie{name:String::from("Interstellar"), description:String::from("Interstellar chronicles the adventures of a group of explorers who make use of a newly discovered wormhole to surpass the limitations on human space travel and conquer the vast distances involved in an interstellar voyage.")}], Some("name")).await.unwrap();
    /// # std::thread::sleep(std::time::Duration::from_secs(1));
    /// #
    /// // retrieve movies (you have to put some movies in the index before)
    /// let movies = movie_index.get_documents::<Movie>(None, None, None).await.unwrap();
    ///
    /// assert!(movies.len() > 0);
    /// # }
    /// ```
    pub async fn get_documents<T: 'static + Document>(
        &self,
        offset: Option<usize>,
        limit: Option<usize>,
        attributes_to_retrieve: Option<&str>,
    ) -> Result<Vec<T>, Error> {
        let mut url = format!("{}/indexes/{}/documents?", self.client.host, self.uid);
        if let Some(offset) = offset {
            url.push_str("offset=");
            url.push_str(offset.to_string().as_str());
            url.push('&');
        }
        if let Some(limit) = limit {
            url.push_str("limit=");
            url.push_str(limit.to_string().as_str());
            url.push('&');
        }
        if let Some(attributes_to_retrieve) = attributes_to_retrieve {
            url.push_str("attributesToRetrieve=");
            url.push_str(attributes_to_retrieve.to_string().as_str());
        }
        Ok(request::<(), Vec<T>>(
            &url,
            self.client.apikey,
            Method::Get,
            200,
        ).await?)
    }

    /// Add a list of [documents](../document/trait.Document.html) or replace them if they already exist.
    ///
    /// If you send an already existing document (same id) the **whole existing document** will be overwritten by the new document.
    /// Fields previously in the document not present in the new document are removed.
    ///
    /// For a partial update of the document see [add_or_update](#method.add_or_update).
    ///
    /// You can use the alias [add_documents](#method.add_documents) if you prefer.
    ///
    /// # Example
    ///
    /// ```
    /// use serde::{Serialize, Deserialize};
    ///
    /// # use meilisearch_sdk::{client::*, indexes::*, document::*};
    /// # use std::thread::sleep;
    /// # use std::time::Duration;
    /// #[derive(Serialize, Deserialize, Debug)]
    /// struct Movie {
    ///    name: String,
    ///    description: String,
    /// }
    /// // that trait is used by the sdk when the primary key is needed
    /// impl Document for Movie {
    ///    type UIDType = String;
    ///    fn get_uid(&self) -> &Self::UIDType {
    ///        &self.name
    ///    }
    /// }
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// let client = Client::new("http://localhost:7700", "masterKey");
    /// let mut movie_index = client.get_or_create("movies").await.unwrap();
    ///
    /// let progress = movie_index.add_or_replace(&[
    ///     Movie{
    ///         name: String::from("Interstellar"),
    ///         description: String::from("Interstellar chronicles the adventures of a group of explorers who make use of a newly discovered wormhole to surpass the limitations on human space travel and conquer the vast distances involved in an interstellar voyage.")
    ///     },
    ///     Movie{
    ///         // note that the id field can only take alphanumerics characters (and '-' and '/')
    ///         name: String::from("MrsDoubtfire"),
    ///         description: String::from("Loving but irresponsible dad Daniel Hillard, estranged from his exasperated spouse, is crushed by a court order allowing only weekly visits with his kids. When Daniel learns his ex needs a housekeeper, he gets the job -- disguised as an English nanny. Soon he becomes not only his children's best pal but the kind of parent he should have been from the start.")
    ///     },
    ///     Movie{
    ///         name: String::from("Apollo13"),
    ///         description: String::from("The true story of technical troubles that scuttle the Apollo 13 lunar mission in 1971, risking the lives of astronaut Jim Lovell and his crew, with the failed journey turning into a thrilling saga of heroism. Drifting more than 200,000 miles from Earth, the astronauts work furiously with the ground crew to avert tragedy.")
    ///     },
    /// ], Some("name")).await.unwrap();
    /// sleep(Duration::from_secs(1)); // MeiliSearch may take some time to execute the request
    /// # progress.get_status().await.unwrap();
    ///
    /// // retrieve movies (you have to put some movies in the index before)
    /// let movies = movie_index.get_documents::<Movie>(None, None, None).await.unwrap();
    /// assert!(movies.len() >= 3);
    /// # }
    /// ```
    pub async fn add_or_replace<T: Document>(
        &'a self,
        documents: &[T],
        primary_key: Option<&str>,
    ) -> Result<Progress<'a>, Error> {
        let url = if let Some(primary_key) = primary_key {
            format!(
                "{}/indexes/{}/documents?primaryKey={}",
                self.client.host, self.uid, primary_key
            )
        } else {
            format!("{}/indexes/{}/documents", self.client.host, self.uid)
        };
        Ok(
            request::<&[T], ProgressJson>(
                &url,
                self.client.apikey,
                Method::Post(documents),
                202,
            ).await?
            .into_progress(self),
        )
    }

    /// Alias for [add_or_replace](#method.add_or_replace).
    pub async fn add_documents<T: Document>(
        &'a self,
        documents: &[T],
        primary_key: Option<&str>,
    ) -> Result<Progress<'a>, Error> {
        self.add_or_replace(documents, primary_key).await
    }

    /// Add a list of documents and update them if they already.
    ///
    /// If you send an already existing document (same id) the old document will be only partially updated according to the fields of the new document.
    /// Thus, any fields not present in the new document are kept and remained unchanged.
    ///
    /// To completely overwrite a document, check out the [add_and_replace documents](#method.add_or_replace) method.
    ///
    /// # Example
    ///
    /// ```
    /// use serde::{Serialize, Deserialize};
    ///
    /// # use meilisearch_sdk::{client::*, indexes::*, document::*};
    /// # use std::thread::sleep;
    /// # use std::time::Duration;
    /// #[derive(Serialize, Deserialize, Debug)]
    /// struct Movie {
    ///    name: String,
    ///    description: String,
    /// }
    /// // that trait is used by the sdk when the primary key is needed
    /// impl Document for Movie {
    ///    type UIDType = String;
    ///    fn get_uid(&self) -> &Self::UIDType {
    ///        &self.name
    ///    }
    /// }
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// let client = Client::new("http://localhost:7700", "masterKey");
    /// let mut movie_index = client.get_or_create("movies").await.unwrap();
    ///
    /// let progress = movie_index.add_or_update(&[
    ///     Movie{
    ///         name: String::from("Interstellar"),
    ///         description: String::from("Interstellar chronicles the adventures of a group of explorers who make use of a newly discovered wormhole to surpass the limitations on human space travel and conquer the vast distances involved in an interstellar voyage.")
    ///     },
    ///     Movie{
    ///         // note that the id field can only take alphanumerics characters (and '-' and '/')
    ///         name: String::from("MrsDoubtfire"),
    ///         description: String::from("Loving but irresponsible dad Daniel Hillard, estranged from his exasperated spouse, is crushed by a court order allowing only weekly visits with his kids. When Daniel learns his ex needs a housekeeper, he gets the job -- disguised as an English nanny. Soon he becomes not only his children's best pal but the kind of parent he should have been from the start.")
    ///     },
    ///     Movie{
    ///         name: String::from("Apollo13"),
    ///         description: String::from("The true story of technical troubles that scuttle the Apollo 13 lunar mission in 1971, risking the lives of astronaut Jim Lovell and his crew, with the failed journey turning into a thrilling saga of heroism. Drifting more than 200,000 miles from Earth, the astronauts work furiously with the ground crew to avert tragedy.")
    ///     },
    /// ], Some("name")).await.unwrap();
    /// sleep(Duration::from_secs(1)); // MeiliSearch may take some time to execute the request
    /// # progress.get_status().await.unwrap();
    ///
    /// // retrieve movies (you have to put some movies in the index before)
    /// let movies = movie_index.get_documents::<Movie>(None, None, None).await.unwrap();
    /// assert!(movies.len() >= 3);
    /// # }
    /// ```
    pub async fn add_or_update<T: Document>(
        &'a self,
        documents: &[T],
        primary_key: Option<&str>,
    ) -> Result<Progress<'a>, Error> {
        let url = if let Some(primary_key) = primary_key {
            format!(
                "{}/indexes/{}/documents?primaryKey={}",
                self.client.host, self.uid, primary_key
            )
        } else {
            format!("{}/indexes/{}/documents", self.client.host, self.uid)
        };
        Ok(
            request::<&[T], ProgressJson>(&url, self.client.apikey, Method::Put(documents), 202).await?
                .into_progress(self),
        )
    }

    /// Delete all documents in the index.
    ///
    /// # Example
    ///
    /// ```
    /// # use serde::{Serialize, Deserialize};
    /// # use meilisearch_sdk::{client::*, indexes::*, document::*};
    /// #
    /// # #[derive(Serialize, Deserialize, Debug)]
    /// # struct Movie {
    /// #    name: String,
    /// #    description: String,
    /// # }
    /// #
    /// # // that trait is used by the sdk when the primary key is needed
    /// # impl Document for Movie {
    /// #    type UIDType = String;
    /// #    fn get_uid(&self) -> &Self::UIDType {
    /// #        &self.name
    /// #    }
    /// # }
    /// #
    /// # #[tokio::main]
    /// # async fn main() {
    /// #
    /// let client = Client::new("http://localhost:7700", "masterKey");
    /// let mut movie_index = client.get_or_create("movies").await.unwrap();
    ///
    /// // add some documents
    ///
    /// let progress = movie_index.delete_all_documents().await.unwrap();
    /// # std::thread::sleep(std::time::Duration::from_secs(1));
    /// # progress.get_status().await.unwrap();
    /// # let movies = movie_index.get_documents::<Movie>(None, None, None).await.unwrap();
    /// # assert_eq!(movies.len(), 0);
    /// # }
    /// ```
    pub async fn delete_all_documents(&'a self) -> Result<Progress<'a>, Error> {
        Ok(request::<(), ProgressJson>(
            &format!("{}/indexes/{}/documents", self.client.host, self.uid),
            self.client.apikey,
            Method::Delete,
            202,
        ).await?
        .into_progress(self))
    }

    /// Delete one document based on its unique id.
    ///
    /// # Example
    ///
    /// ```
    /// # use serde::{Serialize, Deserialize};
    /// # use meilisearch_sdk::{client::*, indexes::*, document::*};
    /// #
    /// # #[derive(Serialize, Deserialize, Debug)]
    /// # struct Movie {
    /// #    name: String,
    /// #    description: String,
    /// # }
    /// #
    /// # // that trait is used by the sdk when the primary key is needed
    /// # impl Document for Movie {
    /// #    type UIDType = String;
    /// #    fn get_uid(&self) -> &Self::UIDType {
    /// #        &self.name
    /// #    }
    /// # }
    /// #
    /// # #[tokio::main]
    /// # async fn main() {
    /// #
    /// let client = Client::new("http://localhost:7700", "masterKey");
    /// let mut movies = client.get_or_create("movies").await.unwrap();
    ///
    /// # movies.add_or_replace(&[Movie{name:String::from("Interstellar"), description:String::from("Interstellar chronicles the adventures of a group of explorers who make use of a newly discovered wormhole to surpass the limitations on human space travel and conquer the vast distances involved in an interstellar voyage.")}], Some("name")).await.unwrap();
    /// # std::thread::sleep(std::time::Duration::from_secs(1));
    /// // add a document with id = Interstellar
    ///
    /// let progress = movies.delete_document("Interstellar").await.unwrap();
    /// # progress.get_status().await.unwrap();
    /// # }
    /// ```
    pub async fn delete_document<T: Display>(&'a self, uid: T) -> Result<Progress<'a>, Error> {
        Ok(request::<(), ProgressJson>(
            &format!(
                "{}/indexes/{}/documents/{}",
                self.client.host, self.uid, uid
            ),
            self.client.apikey,
            Method::Delete,
            202,
        ).await?
        .into_progress(self))
    }

    /// Delete a selection of documents based on array of document id's.
    ///
    /// # Example
    ///
    /// ```
    /// # use serde::{Serialize, Deserialize};
    /// # use meilisearch_sdk::{client::*, indexes::*, document::*};
    /// #
    /// # #[derive(Serialize, Deserialize, Debug)]
    /// # struct Movie {
    /// #    name: String,
    /// #    description: String,
    /// # }
    /// #
    /// # // that trait is used by the sdk when the primary key is needed
    /// # impl Document for Movie {
    /// #    type UIDType = String;
    /// #    fn get_uid(&self) -> &Self::UIDType {
    /// #        &self.name
    /// #    }
    /// # }
    /// #
    /// # #[tokio::main]
    /// # async fn main() {
    /// #
    /// let client = Client::new("http://localhost:7700", "masterKey");
    /// let mut movies = client.get_or_create("movies").await.unwrap();
    ///
    /// // add some documents
    /// # movies.add_or_replace(&[Movie{name:String::from("Interstellar"), description:String::from("Interstellar chronicles the adventures of a group of explorers who make use of a newly discovered wormhole to surpass the limitations on human space travel and conquer the vast distances involved in an interstellar voyage.")},Movie{name:String::from("Unknown"), description:String::from("Unknown")}], Some("name")).await.unwrap();
    /// # std::thread::sleep(std::time::Duration::from_secs(1));
    ///
    /// // delete some documents
    /// let progress = movies.delete_documents(&["Interstellar", "Unknown"]).await.unwrap();
    /// # progress.get_status().await.unwrap();
    /// # }
    /// ```
    pub async fn delete_documents<T: Display + Serialize + std::fmt::Debug>(
        &'a self,
        uids: &[T],
    ) -> Result<Progress<'a>, Error> {
        Ok(request::<&[T], ProgressJson>(
            &format!(
                "{}/indexes/{}/documents/delete-batch",
                self.client.host, self.uid
            ),
            self.client.apikey,
            Method::Post(uids),
            202,
        ).await?
        .into_progress(self))
    }

    /// Alias for the [update method](#method.update).
    pub async fn set_primary_key(&self, primary_key: &str) -> Result<(), Error> {
        self.update(primary_key).await
    }

    /// Get stats of an index.
    ///
    /// # Example
    ///
    /// ```
    /// # use meilisearch_sdk::{client::*, indexes::*};
    /// #
    /// # #[tokio::main]
    /// # async fn main() {
    /// let client = Client::new("http://localhost:7700", "masterKey");
    /// let movies = client.get_or_create("movies").await.unwrap();
    ///
    /// let stats = movies.get_stats().await.unwrap();
    /// # }
    /// ```
    pub async fn get_stats(&self) -> Result<IndexStats, Error> {
        request::<serde_json::Value, IndexStats>(
            &format!("{}/indexes/{}/stats", self.client.host, self.uid),
            self.client.apikey,
            Method::Get,
            200,
        ).await
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IndexStats {
    pub number_of_documents: usize,
    pub is_indexing: bool,
    pub fields_distribution: HashMap<String, usize>,
}
