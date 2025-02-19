// Copyright (C) 2019-2023 Aleo Systems Inc.
// This file is part of the snarkVM library.

// The snarkVM library is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// The snarkVM library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with the snarkVM library. If not, see <https://www.gnu.org/licenses/>.

mod literal_type;
pub use literal_type::LiteralType;

mod plaintext_type;
pub use plaintext_type::PlaintextType;

mod record_type;
pub use record_type::{EntryType, RecordType};

mod register_type;
pub use register_type::RegisterType;

mod struct_;
pub use struct_::Struct;

mod value_type;
pub use value_type::ValueType;
