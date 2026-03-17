melody = \relative c'' {
  \clef treble
  \key f \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  g2 a2 f2 d4 d4 e4 g4 fis2 g2 r2 \break

  % Line 2
  bes2 bes4 c4 d4 d4 c4 bes4 a2 g2 r2 \break

  % Line 3
  d2 g2 bes2 a4 g4 f4 f4 e2 d2 r2 \break

  % Line 4
  g2 bes2 a2 g4 f4 g4 bes2 a4 bes2 r2 \break

  % Line 5
  d2 bes4 c4 g4 bes4 a2 g2 f2 r2 \break

  % Line 6
  bes2 a4 g4 f4 g4 g4 fis4 g1 \bar "|."
}
