melody = \relative c' {
  \clef treble
  \key bes \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  d4 g4 g4 a4 a4 bes4.( a8) g4 \break

  % Line 2
  g4 c4 c4 bes4 bes4 a2. \break

  % Line 3
  d,4 g4 g4 a4 a4 bes4.( c8) d4 \break

  % Line 4
  c4 bes4 g4 a4 fis4 g2. \break

  % Line 5
  d'4 d4 bes4 a4 c4 bes2 a4 \break

  % Line 6
  g4 fis4 g4 c4 bes4 a2. \break

  % Line 7
  d,4 g4 g4 a4 a4 bes4.( c8) d4 \break

  % Line 8
  c4 bes4 g4 a4 fis4 g2. \bar "|."
}
