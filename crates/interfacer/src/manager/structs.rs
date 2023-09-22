/*
 * Copyright (C) 2023 James Draycott <me@racci.dev>
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, version 3.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

pub mod client {
    use crate::hudu::web::Hudu;
    use crate::nable::web::NAble;
    use crate::Client;
    use async_trait::async_trait;

    pub enum ManagementType {
        Billable,
        Managed,
        Services,
        Unknown,
    }

    pub struct ClientGrouper {
        pub name: String,
        pub management_type: ManagementType,
        pub hudu: crate::hudu::structs::company::Company,
        pub nable: crate::nable::structs::client::RMMClient,
    }

    #[async_trait]
    pub trait ClientFinder {
        async fn find(&self, client: Client) -> ClientGrouper
        where
            Self: Sized,
            Client: Hudu + NAble;
    }
}
